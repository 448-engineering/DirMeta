use crate::{CowStr, FsUtils};
use async_fs::{read_dir, ReadDir};
use async_recursion::async_recursion;
use file_format::FileFormat;
use futures_lite::{
    io::{self, ErrorKind},
    stream::StreamExt,
};
use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};
use tai64::Tai64N;

#[cfg(feature = "time")]
use crate::DateTimeString;

#[derive(Debug, PartialEq, Eq, Default, Clone)]
pub struct DirMetadata<'a> {
    name: CowStr<'a>,
    path: PathBuf,
    directories: Vec<PathBuf>,
    files: Vec<FileMetadata<'a>>,
    size: usize,
    errors: Vec<DirError<'a>>,
}

impl<'a> DirMetadata<'a> {
    pub fn new(path: &'a str) -> Self {
        let dir_name: PathBuf = path.into();
        let dir_name = dir_name.file_name();

        let name = match dir_name {
            Some(name) => CowStr::Owned(name.to_string_lossy().to_string()),
            None => path.into(),
        };

        DirMetadata {
            path: path.into(),
            name,
            ..Default::default()
        }
    }

    /// Returns an error if the directory cannot be accessed
    pub async fn dir_metadata(mut self) -> Result<Self, io::Error> {
        let mut dir = read_dir(&self.path).await?;

        self.iter_dir(&mut dir).await;

        Ok(self)
    }

    #[async_recursion]
    pub async fn iter_dir(&mut self, prepared_dir: &mut ReadDir) -> &mut Self {
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

                        let format = match FileFormat::from_file(entry.path()) {
                            Ok(format_detected) => format_detected,
                            Err(_) => FileFormat::default(),
                        };
                        file_meta.file_format = format;

                        file_meta.name =
                            CowStr::Owned(entry.file_name().to_string_lossy().to_string());
                        file_meta.path = entry.path();
                        match entry.metadata().await {
                            Ok(meta) => {
                                let current_file_size = meta.len() as usize;
                                self.size += current_file_size;
                                file_meta.size = current_file_size;
                                file_meta.accessed = FsUtils::maybe_time(meta.accessed().ok());
                                file_meta.modified = FsUtils::maybe_time(meta.modified().ok());
                                file_meta.created = FsUtils::maybe_time(meta.created().ok());
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
                    self.iter_dir(&mut prepared_dir).await;
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

    pub fn dir_name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn dir_path(&self) -> &Path {
        self.path.as_ref()
    }

    pub fn directories(&self) -> &[PathBuf] {
        self.directories.as_ref()
    }

    pub fn files(&self) -> &[FileMetadata<'a>] {
        self.files.as_ref()
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn size_formatted(&self) -> String {
        FsUtils::size_to_bytes(self.size)
    }

    pub fn errors(&self) -> &[DirError<'a>] {
        self.errors.as_ref()
    }
}

#[derive(Debug, PartialEq, Eq, Default, Clone)]
pub struct FileMetadata<'a> {
    name: CowStr<'a>,
    path: PathBuf,
    size: usize,
    read_only: bool,
    created: Option<Tai64N>,
    accessed: Option<Tai64N>,
    modified: Option<Tai64N>,
    symlink: bool,
    file_format: FileFormat,
}

impl<'a> FileMetadata<'a> {
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn path(&self) -> &Path {
        self.path.as_ref()
    }
    pub fn size(&self) -> usize {
        self.size
    }

    pub fn formatted_size(&self) -> String {
        FsUtils::size_to_bytes(self.size)
    }

    pub fn accessed(&self) -> Option<Tai64N> {
        self.accessed
    }

    #[cfg(feature = "time")]
    pub fn accessed_24hr(&self) -> Option<DateTimeString<'a>> {
        Some(FsUtils::tai64_to_local_hrs(&self.accessed?))
    }

    #[cfg(feature = "time")]
    pub fn accessed_am_pm(&self) -> Option<DateTimeString<'a>> {
        Some(FsUtils::tai64_to_local_am_pm(&self.accessed?))
    }

    #[cfg(feature = "time")]
    pub fn accessed_humatime(&self) -> Option<String> {
        FsUtils::tai64_now_duration_to_humantime(&self.accessed?)
    }

    #[cfg(feature = "time")]
    pub fn modified_24hr(&self) -> Option<DateTimeString<'a>> {
        Some(FsUtils::tai64_to_local_hrs(&self.modified?))
    }

    #[cfg(feature = "time")]
    pub fn modified_am_pm(&self) -> Option<DateTimeString<'a>> {
        Some(FsUtils::tai64_to_local_am_pm(&self.modified?))
    }

    #[cfg(feature = "time")]
    pub fn modified_humatime(&self) -> Option<String> {
        FsUtils::tai64_now_duration_to_humantime(&self.modified?)
    }

    #[cfg(feature = "time")]
    pub fn created_24hr(&self) -> Option<DateTimeString<'a>> {
        Some(FsUtils::tai64_to_local_hrs(&self.created?))
    }

    #[cfg(feature = "time")]
    pub fn created_am_pm(&self) -> Option<DateTimeString<'a>> {
        Some(FsUtils::tai64_to_local_am_pm(&self.created?))
    }

    #[cfg(feature = "time")]
    pub fn created_humatime(&self) -> Option<String> {
        FsUtils::tai64_now_duration_to_humantime(&self.created?)
    }

    pub fn read_only(&self) -> bool {
        self.read_only
    }
    pub fn symlink(&self) -> bool {
        self.symlink
    }

    pub fn file_format(&self) -> &FileFormat {
        &self.file_format
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct DirError<'a> {
    path: PathBuf,
    error: ErrorKind,
    display: CowStr<'a>,
}
