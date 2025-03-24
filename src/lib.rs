#![deny(unsafe_code)]
#![forbid(missing_docs)]
#![doc = include_str!("../README.md")]

mod utils;
pub use utils::*;

mod fs;
pub use fs::*;

#[cfg(all(not(doc), feature = "sync", feature = "async"))]
compile_error!("Features 'sync' and 'async' cannot be enabled at the same time.");

#[cfg(all(not(doc), feature = "sync", feature = "watcher"))]
compile_error!("`watcher` feature can only be compiled with `async` feature.");

#[cfg(feature = "watcher")]
mod watcher;
/// This directory inherits most types from `inotify` crate
#[cfg(feature = "watcher")]
pub use watcher::*;

#[cfg(feature = "async")]
pub use async_recursion;

#[cfg(feature = "size")]
pub use byte_prefix;

#[cfg(feature = "time")]
pub use chrono;

#[cfg(feature = "file-type")]
pub use file_format;

#[cfg(feature = "time")]
pub use humantime;

#[cfg(feature = "watcher")]
pub use inotify;

#[cfg(feature = "watcher")]
pub use async_channel;

#[cfg(feature = "async")]
pub use async_io;

#[cfg(feature = "async")]
pub use futures_lite;

#[cfg(feature = "async")]
pub use blocking;

#[cfg(feature = "time")]
pub use tai64;
