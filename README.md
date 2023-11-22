### FsMeta
This crate adds the ability to recursively walk through a directory in an async fashion. The result is metadata about the directory which currently includes:

- [x] Get the total size of the directory in bytes
- [x] Get the size of files
- [x] Formatting of the file and directory sizes in human readable format (B, KiB, MiB, ...)
- [x] Fetch all the directories
- [x] Fetch all the files 
- [x] Fetch the created, assessed and modified timestamps in Tai64N (monotonic, no leap seconds) timstamps
- [x] Fetch the created, assessed and modified timestamps in Local time (24 hour / 12 hour format)
- [x] format timestamps according to duration
- [x] Get the file format eg PDF or plain text
- [x] Returns all the files and directories in current directory with any errors that occur instead of just returning the error when error is encountered (like `fs::read_dir()`)
- [ ] Use parallelism where applicable (TODO)



#### Examples
```toml
[dependencies] 
dir-meta = {version = "*", default-features = false} #deactivate methods for converting timestamps to human readable formats in local time setting `default-features` to `false`
```

```rust
smol::block_on(async {
            // Read a directory
            let outcome = dir_meta::DirMetadata::new("src").dir_metadata().await.unwrap();

            dbg!(&outcome);

            // Get size of directory formatted as human readable
            dbg!(outcome.size_formatted());

            // Iterate over the files
            
            for file in outcome.files() {
                dbg!(&file.name()); //Get file name
                dbg!(&file.accessed_24hr()); // Get last accessed time in 24 hour format
                dbg!(file.accessed_am_pm()); //Get last accessed time in 12 hour format
                dbg!(&file.accessed_humatime()); //Get last accessed time based on duration since current time
                dbg!(&file.created_24hr());  //Get last created time in 24 hour format
                dbg!(&file.created_am_pm()); //Get last created time in 24 hour format
                dbg!(&file.created_humatime()); //Get last created time based on duration since current time
                dbg!(&file.modified_24hr()); //Get last modified time in 24 hour format
                dbg!(&file.modified_am_pm()); //Get last modified time in 24 hour format
                dbg!(&file.modified_humatime()); //Get last modified time based on duration since current time
                dbg!(file.formatted_size()); // Get the size of the file in human formatted size 
                dbg!(file.file_format()); // Get the format of the file eg (PDF)
            }
            
        })
```

##### LICENSE
The code is licensed under APACHE-2.0

##### Code of Conduct
All contributions must obey the rules in the Rust Code of Conduct by the Rust Foundation