use crate::CowStr;

use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

#[cfg(feature = "async")]
use async_recursion::async_recursion;

#[cfg(feature = "async")]
use futures_lite::StreamExt;

#[cfg(feature = "file-type")]
use file_format::FileFormat;

#[cfg(feature = "time")]
use tai64::Tai64N;

#[cfg(feature = "time")]
use crate::DateTimeString;

/// The Metadata of all directories and files in the current directory
/// #### Example
/// ```rust
/// use dir_meta::DirMetadata;
///
/// // With feature `async` enabled using `cargo add dir-meta --features async`
/// let dir = DirMetadata::new("/path/to/directory").async_dir_metadata();
/// ```
#[derive(Debug, PartialEq, Eq, Default, Clone)]
pub struct DirMetadata<'a> {
    name: CowStr<'a>,
    path: PathBuf,
    directories: Vec<PathBuf>,
    files: Vec<FileMetadata<'a>>,
    #[cfg(feature = "extra")]
    size: usize,
    errors: Vec<DirError<'a>>,
}

impl<'a> DirMetadata<'_> {
    /// Create a new instance of [Self]
    /// but with the path as a `&str`
    pub fn new(path: &'a str) -> Self {
        Self::new_path_buf(path.into())
    }

    /// Create a new instance of [Self]
    pub fn new_path_buf(path: PathBuf) -> Self {
        let name = Cow::Owned(path.file_name().unwrap().to_string_lossy().to_string());

        Self {
            path,
            name,
            ..Default::default()
        }
    }

    /// Multiple files can have the same name if they are in different dirs
    /// so using this method returns a [vector](Vec) of [FileMetadata]
    pub fn get_file(&self, file_name: &'a str) -> Vec<&'a FileMetadata> {
        self.files()
            .iter()
            .filter(|file| file.name() == file_name)
            .collect()
    }

    /// Get a file by it's absolute path (from root)
    pub fn get_file_by_path(&self, path: &'a str) -> Option<&'a FileMetadata> {
        self.files()
            .iter()
            .find(|file| file.path() == Path::new(path))
    }

    /// Returns an error if the directory cannot be accessed
    /// Read all the directories and files in the given path in async fashion
    #[cfg(feature = "async")]
    pub async fn async_dir_metadata(mut self) -> Result<Self, std::io::Error> {
        use async_fs::read_dir;

        let mut dir = read_dir(&self.path).await?;

        self.async_iter_dir(&mut dir).await;

        Ok(self)
    }

    /// Returns an error if the directory cannot be accessed
    /// Read all the directories and files in the given path
    #[cfg(feature = "sync")]
    pub fn sync_dir_metadata(mut self) -> Result<Self, std::io::Error> {
        use std::fs::read_dir;
        let mut dir = read_dir(&self.path)?;

        self.sync_iter_dir(&mut dir);

        Ok(self)
    }

    /// Recursively iterate over directories inside directories
    #[cfg(feature = "async")]
    #[async_recursion]
    pub async fn async_iter_dir(
        &'a mut self,
        prepared_dir: &mut async_fs::ReadDir,
    ) -> &'a mut Self {
        use async_fs::read_dir;

        let mut directories = Vec::<PathBuf>::new();

        while let Some(entry_result) = prepared_dir.next().await {
            match entry_result {
                Err(error) => {
                    self.errors.push(DirError {
                        path: self.path.clone(),
                        error: error.kind(),
                        display: error.to_string().into(),
                    });
                }
                Ok(entry) => {
                    let mut is_dir = false;

                    match entry.file_type().await {
                        Ok(file_type) => is_dir = file_type.is_dir(),
                        Err(error) => {
                            let inner_path = entry.path();

                            self.errors.push(DirError {
                                path: inner_path.clone(),
                                error: error.kind(),
                                display: Cow::Owned(format!(
                                    "Unable to check if `{}` is a directory",
                                    inner_path.display()
                                )),
                            });
                        }
                    }

                    if is_dir {
                        directories.push(entry.path())
                    } else {
                        let mut file_meta = FileMetadata::default();

                        #[cfg(all(feature = "file-type", feature = "async"))]
                        {
                            let cloned_path = entry.path().clone();
                            let get_file_format =
                                blocking::unblock(move || FileFormat::from_file(cloned_path));
                            let format = (get_file_format.await).unwrap_or_default();
                            file_meta.file_format = format;
                        }

                        file_meta.name =
                            CowStr::Owned(entry.file_name().to_string_lossy().to_string());
                        file_meta.path = entry.path();

                        #[cfg(any(feature = "size", feature = "time", feature = "extra"))]
                        match entry.metadata().await {
                            Ok(meta) => {
                                #[cfg(feature = "extra")]
                                {
                                    let current_file_size = meta.len() as usize;
                                    self.size += current_file_size;
                                    file_meta.size = current_file_size;
                                }

                                #[cfg(feature = "time")]
                                {
                                    file_meta.accessed =
                                        crate::FsUtils::maybe_time(meta.accessed().ok());
                                    file_meta.modified =
                                        crate::FsUtils::maybe_time(meta.modified().ok());
                                    file_meta.created =
                                        crate::FsUtils::maybe_time(meta.created().ok());
                                }
                            }
                            Err(error) => {
                                self.errors.push(DirError {
                                    path: entry.path(),
                                    error: error.kind(),
                                    display: Cow::Owned(format!(
                                        "Unable to access metadata of file `{}`",
                                        entry.path().display()
                                    )),
                                });
                            }
                        }

                        self.files.push(file_meta);
                    }
                }
            }
        }

        let mut dir_iter = futures_lite::stream::iter(&directories);

        while let Some(path) = dir_iter.next().await {
            match read_dir(path.clone()).await {
                Ok(mut prepared_dir) => {
                    self.async_iter_dir(&mut prepared_dir).await;
                }
                Err(error) => self.errors.push(DirError {
                    path: path.to_owned(),
                    error: error.kind(),
                    display: Cow::Owned(format!(
                        "Unable to access metadata of file `{}`",
                        path.display()
                    )),
                }),
            }
        }

        self.directories.extend_from_slice(&directories);

        self
    }

    /// Recursively iterate over directories inside directories
    #[cfg(feature = "sync")]
    pub fn sync_iter_dir(&mut self, prepared_dir: &mut std::fs::ReadDir) -> &mut Self {
        let mut directories = Vec::<PathBuf>::new();

        prepared_dir
            .by_ref()
            .for_each(|entry_result| match entry_result {
                Err(error) => {
                    self.errors.push(DirError {
                        path: self.path.clone(),
                        error: error.kind(),
                        display: error.to_string().into(),
                    });
                }
                Ok(entry) => {
                    let mut is_dir = false;

                    match entry.file_type() {
                        Ok(file_type) => is_dir = file_type.is_dir(),
                        Err(error) => {
                            let inner_path = entry.path();

                            self.errors.push(DirError {
                                path: inner_path.clone(),
                                error: error.kind(),
                                display: Cow::Owned(format!(
                                    "Unable to check if `{}` is a directory",
                                    inner_path.display()
                                )),
                            });
                        }
                    }

                    if is_dir {
                        directories.push(entry.path())
                    } else {
                        let mut file_meta = FileMetadata::default();

                        #[cfg(all(feature = "file-type", feature = "sync"))]
                        {
                            let cloned_path = entry.path().clone();
                            let get_file_format = FileFormat::from_file(cloned_path);
                            let format = (get_file_format).unwrap_or_default();
                            file_meta.file_format = format;
                        }

                        file_meta.name =
                            CowStr::Owned(entry.file_name().to_string_lossy().to_string());
                        file_meta.path = entry.path();
                        #[cfg(any(feature = "size", feature = "time", feature = "extra"))]
                        match entry.metadata() {
                            Ok(meta) => {
                                #[cfg(feature = "extra")]
                                {
                                    let current_file_size = meta.len() as usize;
                                    self.size += current_file_size;
                                    file_meta.size = current_file_size;
                                }

                                #[cfg(feature = "time")]
                                {
                                    file_meta.accessed =
                                        crate::FsUtils::maybe_time(meta.accessed().ok());
                                    file_meta.modified =
                                        crate::FsUtils::maybe_time(meta.modified().ok());
                                    file_meta.created =
                                        crate::FsUtils::maybe_time(meta.created().ok());
                                }
                            }
                            Err(error) => {
                                self.errors.push(DirError {
                                    path: entry.path(),
                                    error: error.kind(),
                                    display: Cow::Owned(format!(
                                        "Unable to access metadata of file `{}`",
                                        entry.path().display()
                                    )),
                                });
                            }
                        }

                        self.files.push(file_meta);
                    }
                }
            });

        directories
            .iter()
            .for_each(|path| match std::fs::read_dir(path.clone()) {
                Ok(mut prepared_dir) => {
                    self.sync_iter_dir(&mut prepared_dir);
                }
                Err(error) => self.errors.push(DirError {
                    path: path.to_owned(),
                    error: error.kind(),
                    display: Cow::Owned(format!(
                        "Unable to access metadata of file `{}`",
                        path.display()
                    )),
                }),
            });

        self.directories.extend_from_slice(&directories);

        self
    }

    /// Get the name of the current directory
    pub fn dir_name(&self) -> &str {
        self.name.as_ref()
    }

    /// Get the path of the current directory
    pub fn dir_path(&self) -> &Path {
        self.path.as_ref()
    }

    /// Get all the sub-directories of the current directory
    pub fn directories(&self) -> &[PathBuf] {
        self.directories.as_ref()
    }

    /// Get all the files in the current directory and all the files in it's sub-directory
    pub fn files(&'a self) -> &'a [FileMetadata<'a>] {
        self.files.as_ref()
    }

    /// Get the size of the directory including the  size of all files in the sub-directories
    #[cfg(feature = "extra")]
    pub fn size(&self) -> usize {
        self.size
    }

    /// Get the size of the directory including the  size of all files in the sub-directories in human readable format
    #[cfg(feature = "size")]
    pub fn size_formatted(&self) -> String {
        crate::FsUtils::size_to_bytes(self.size)
    }

    /// Get all the errors encountered while opening the sub-directories and files
    pub fn errors(&'a self) -> &'a [DirError<'a>] {
        self.errors.as_ref()
    }
}

/// The file metadata like file name, file type, file size, file path etc
#[derive(Debug, PartialEq, Eq, Default, Clone)]
pub struct FileMetadata<'a> {
    name: CowStr<'a>,
    path: PathBuf,
    #[cfg(feature = "extra")]
    size: usize,
    #[cfg(feature = "extra")]
    read_only: bool,
    #[cfg(feature = "time")]
    created: Option<Tai64N>,
    #[cfg(feature = "time")]
    accessed: Option<Tai64N>,
    #[cfg(feature = "time")]
    modified: Option<Tai64N>,
    #[cfg(feature = "extra")]
    symlink: bool,
    #[cfg(feature = "file-type")]
    file_format: FileFormat,
}

impl<'a> FileMetadata<'a> {
    /// Get the name of the file
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Get the path of the file
    pub fn path(&self) -> &Path {
        self.path.as_ref()
    }

    /// Get the size of the file
    #[cfg(feature = "extra")]
    pub fn size(&self) -> usize {
        self.size
    }

    /// Get the size of the file in human readable format
    #[cfg(feature = "size")]
    pub fn formatted_size(&self) -> String {
        crate::FsUtils::size_to_bytes(self.size)
    }

    /// Get the TAI64N timestamp when the file was last accessed
    #[cfg(feature = "time")]
    pub fn accessed(&self) -> Option<Tai64N> {
        self.accessed
    }

    /// Get the TAI64N timestamp when the file was last modified
    #[cfg(feature = "time")]
    pub fn modified(&self) -> Option<Tai64N> {
        self.modified
    }

    /// Get the TAI64N timestamp when the file was last created
    #[cfg(feature = "time")]
    pub fn created(&self) -> Option<Tai64N> {
        self.created
    }

    /// Get the timestamp in local time in 24 hour format when the file was last accessed
    #[cfg(feature = "time")]
    pub fn accessed_24hr(&self) -> Option<DateTimeString<'a>> {
        Some(crate::FsUtils::tai64_to_local_hrs(&self.accessed?))
    }

    /// Get the timestamp in local time in 12 hour format when the file was last accessed
    #[cfg(feature = "time")]
    pub fn accessed_am_pm(&self) -> Option<DateTimeString<'a>> {
        Some(crate::FsUtils::tai64_to_local_am_pm(&self.accessed?))
    }

    /// Get the time passed since access of a file eg `3 sec ago`
    #[cfg(feature = "time")]
    pub fn accessed_humatime(&self) -> Option<String> {
        crate::FsUtils::tai64_now_duration_to_humantime(&self.accessed?)
    }

    /// Get the timestamp in local time in 24 hour format when the file was last modified
    #[cfg(feature = "time")]
    pub fn modified_24hr(&self) -> Option<DateTimeString<'a>> {
        Some(crate::FsUtils::tai64_to_local_hrs(&self.modified?))
    }

    /// Get the timestamp in local time in 12 hour format when the file was last modified
    #[cfg(feature = "time")]
    pub fn modified_am_pm(&self) -> Option<DateTimeString<'a>> {
        Some(crate::FsUtils::tai64_to_local_am_pm(&self.modified?))
    }

    /// Get the time passed since modification of a file eg `3 sec ago`
    #[cfg(feature = "time")]
    pub fn modified_humatime(&self) -> Option<String> {
        crate::FsUtils::tai64_now_duration_to_humantime(&self.modified?)
    }

    /// Get the timestamp in local time in 24 hour format when the file was created
    #[cfg(feature = "time")]
    pub fn created_24hr(&self) -> Option<DateTimeString<'a>> {
        Some(crate::FsUtils::tai64_to_local_hrs(&self.created?))
    }

    /// Get the timestamp in local time in 12 hour format when the file was created
    #[cfg(feature = "time")]
    pub fn created_am_pm(&self) -> Option<DateTimeString<'a>> {
        Some(crate::FsUtils::tai64_to_local_am_pm(&self.created?))
    }

    /// Get the time passed since file was created of a file eg `3 sec ago`
    #[cfg(feature = "time")]
    pub fn created_humatime(&self) -> Option<String> {
        crate::FsUtils::tai64_now_duration_to_humantime(&self.created?)
    }

    /// Is the file read only
    #[cfg(feature = "extra")]
    pub fn read_only(&self) -> bool {
        self.read_only
    }

    /// Is the file a symbolic link
    #[cfg(feature = "extra")]
    pub fn symlink(&self) -> bool {
        self.symlink
    }

    /// Get the format of the current file
    #[cfg(feature = "file-type")]
    pub fn file_format(&self) -> &FileFormat {
        &self.file_format
    }
}

/// An error encountered while accessing a file or sub-directory
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct DirError<'a> {
    /// The path to the sub-directory or file where the error occurred
    pub path: PathBuf,
    /// The kind of error that occurred based on [std::io::ErrorKind]
    pub error: std::io::ErrorKind,
    /// The formatted error as a [String]
    pub display: CowStr<'a>,
}

#[cfg(test)]
mod sanity_checks {

    #[cfg(all(feature = "async", feature = "size", feature = "extra"))]
    #[test]
    fn async_features() {
        smol::block_on(async {
            let outcome = crate::DirMetadata::new("src")
                .async_dir_metadata()
                .await
                .unwrap();

            {
                #[cfg(feature = "time")]
                for file in outcome.files() {
                    assert_ne!("", file.name());
                    assert_ne!(Option::None, file.accessed_24hr());
                    assert_ne!(Option::None, file.accessed_am_pm());
                    assert_ne!(Option::None, file.accessed_humatime());
                    assert_ne!(Option::None, file.created_24hr());
                    assert_ne!(Option::None, file.created_am_pm());
                    assert_ne!(Option::None, file.created_humatime());
                    assert_ne!(Option::None, file.modified_24hr());
                    assert_ne!(Option::None, file.modified_am_pm());
                    assert_ne!(Option::None, file.modified_humatime());
                    assert_ne!(String::default(), file.formatted_size());
                }
            }
        })
    }

    #[cfg(all(feature = "sync", feature = "size", feature = "extra"))]
    #[test]
    fn sync_features() {
        use file_format::FileFormat;

        smol::block_on(async {
            let outcome = crate::DirMetadata::new("src").sync_dir_metadata().unwrap();

            {
                #[cfg(feature = "time")]
                for file in outcome.files() {
                    assert_ne!("", file.name());
                    assert_ne!(Option::None, file.accessed_24hr());
                    assert_ne!(Option::None, file.accessed_am_pm());
                    assert_ne!(Option::None, file.accessed_humatime());
                    assert_ne!(Option::None, file.created_24hr());
                    assert_ne!(Option::None, file.created_am_pm());
                    assert_ne!(Option::None, file.created_humatime());
                    assert_ne!(Option::None, file.modified_24hr());
                    assert_ne!(Option::None, file.modified_am_pm());
                    assert_ne!(Option::None, file.modified_humatime());
                    assert_ne!(String::default(), file.formatted_size());
                }
            }

            #[cfg(feature = "extra")]
            {
                assert!(outcome.size() > 0usize);
            }

            #[cfg(feature = "file-type")]
            {
                let path = String::from(env!("CARGO_MANIFEST_DIR")) + "/src";
                let file = outcome.get_file_by_path(&path);
                assert!(file.is_some());
                assert_eq!(file.unwrap().file_format(), &FileFormat::PlainText);
            }
        })
    }
}
