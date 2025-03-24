use async_channel::Sender;

use inotify::{EventMask, Inotify, WatchMask};
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

/// The error that a channel has been closed
pub const SENDER_CHANNEL_ERROR: &str = "SENDER_CHANNEL_CLOSED";

/// The sender type for a channel as a type for reusability
pub type FsSender = Sender<WatcherOutcome>;

/// Create a watcher for a certain path that can be a file or directory
///
/// #### Structure
/// ```rust
/// use dir_meta::FsSender;
/// use std::path::PathBuf;
///
/// #[derive(Debug)]
/// pub struct FsWatcher {
///     path: Option<PathBuf>,
///     sender: FsSender,
/// }
/// ```
///
/// #### Example
/// ```rust
/// use dir_meta::{inotify::WatchMask, smol::channel, FsWatcher, WatcherOutcome};
///
/// smol::block_on(async {
///     let (sender, receiver) = channel::unbounded::<WatcherOutcome>();
///
///     let watch_options =
///         WatchMask::MODIFY | WatchMask::CREATE | WatchMask::DELETE | WatchMask::DELETE_SELF;
///
///     smol::spawn(FsWatcher::new(sender).path("Foo").watch(watch_options)).detach();
///
///     while let Ok(data) = receiver.recv().await {
///         dbg!(data);
///     }
/// });
/// ```
#[derive(Debug)]
pub struct FsWatcher {
    path: Option<PathBuf>, //Option is used here to make it easier to return ErrorKind::NotFound in io::Result when calling watcher
    sender: FsSender,
}

impl FsWatcher {
    /// Create a new [FsWatcher] by passing an async-channel::channel::Sender with type specified by [FsSender]
    pub fn new(sender: FsSender) -> Self {
        Self {
            sender,
            path: Option::default(),
        }
    }

    /// Add the path to listen to
    pub fn path(mut self, path: impl AsRef<Path>) -> Self {
        self.path.replace(path.as_ref().to_path_buf());

        self
    }

    /// Watch the path using the parameters from `inotify::WatchMask`
    /// which can be concatenated `WatchMask::MODIFY | WatchMask::CREATE | WatchMask::DELETE`
    pub async fn watch(self, watch_for: WatchMask) -> futures_lite::io::Result<()> {
        if let Some(path) = self.path {
            let mut inotify = Inotify::init()?;

            inotify.watches().add(&path, watch_for)?;

            //TODO add logging here "Watching current directory for activity..."

            let mut buffer = [0u8; 4096];

            loop {
                let events = inotify.read_events_blocking(&mut buffer)?;

                for event in events {
                    let outcome: WatcherOutcome = event.into();

                    if self.sender.clone().send(outcome).await.is_err() {
                        return Err(futures_lite::io::Error::new(
                            futures_lite::io::ErrorKind::Other,
                            SENDER_CHANNEL_ERROR,
                        ));
                    }
                }
            }
        } else {
            Err(futures_lite::io::Error::new(
                futures_lite::io::ErrorKind::NotFound,
                "The path was not found, maybe you didn't specify it",
            ))
        }
    }
}

/// Events triggered from watching a directory or file
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum WatcherEvents {
    /// File was accessed
    ///
    /// When watching a directory, this event is only triggered for objects
    /// inside the directory, not the directory itself.
    Access,
    /// Metadata (permissions, timestamps, ...) changed
    ///
    /// When watching a directory, this event can be triggered for the
    /// directory itself, as well as objects inside the directory.
    Attrib,
    /// File opened for writing was closed
    ///
    /// When watching a directory, this event is only triggered for objects
    /// inside the directory, not the directory itself.
    CloseWrite,
    /// File or directory not opened for writing was closed
    ///
    /// When watching a directory, this event can be triggered for the
    /// directory itself, as well as objects inside the directory.
    CloseNoWrite,
    /// File/directory created in watched directory
    ///
    /// When watching a directory, this event is only triggered for objects
    /// inside the directory, not the directory itself.
    Create,
    /// File/directory deleted from watched directory
    ///
    /// When watching a directory, this event is only triggered for objects
    /// inside the directory, not the directory itself.
    Delete,
    /// Watched file/directory was deleted
    DeleteSelf,
    /// File was modified
    ///
    /// When watching a directory, this event is only triggered for objects
    /// inside the directory, not the directory itself.
    Modify,
    /// Watched file/directory was moved
    MoveSelf,
    /// File was renamed/moved; watched directory contained old name
    ///
    /// When watching a directory, this event is only triggered for objects
    /// inside the directory, not the directory itself.
    MovedFrom,
    /// File was renamed/moved; watched directory contains new name
    ///
    /// When watching a directory, this event is only triggered for objects
    /// inside the directory, not the directory itself.
    MovedTo,
    /// File or directory was opened
    ///
    /// When watching a directory, this event can be triggered for the
    /// directory itself, as well as objects inside the directory.
    Open,
    /// Watch was removed
    ///
    /// This event will be generated, if the watch was removed explicitly
    /// (via [`inotify::Watches::remove`]), or automatically (because the file was
    /// deleted or the file system was unmounted).
    Ignored,
    /// Event related to a directory
    ///
    /// The subject of the event is a directory.
    IsDir,
    /// Event queue overflowed
    ///
    /// The event queue has overflowed and events have presumably been lost.
    QueueOverflow,
    /// File system containing watched object was unmounted.
    /// File system was unmounted
    ///
    /// The file system that contained the watched object has been
    /// unmounted. An event with [`EventMask::IGNORED`] will subsequently be
    /// generated for the same watch descriptor.
    Unmount,
    /// Current event is unsupported
    Unsupported,
}

impl From<EventMask> for WatcherEvents {
    fn from(value: EventMask) -> Self {
        match value {
            EventMask::ACCESS => Self::Access,
            EventMask::ATTRIB => Self::Attrib,
            EventMask::CLOSE_WRITE => Self::CloseWrite,
            EventMask::CLOSE_NOWRITE => Self::CloseNoWrite,
            EventMask::CREATE => Self::Create,
            EventMask::DELETE => Self::Delete,
            EventMask::DELETE_SELF => Self::DeleteSelf,
            EventMask::MODIFY => Self::Modify,
            EventMask::MOVE_SELF => Self::MoveSelf,
            EventMask::MOVED_FROM => Self::MovedFrom,
            EventMask::MOVED_TO => Self::MovedTo,
            EventMask::OPEN => Self::Open,
            EventMask::IGNORED => Self::Ignored,
            EventMask::ISDIR => Self::IsDir,
            EventMask::Q_OVERFLOW => Self::QueueOverflow,
            EventMask::UNMOUNT => Self::Unmount,
            _ => Self::Unsupported,
        }
    }
}

/// The outcome of a watched file or directory
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct WatcherOutcome {
    /// Identifies the watch this event originates from
    /// This WatchDescriptor is equal to the one that Watches::add returned when interest for this event was registered. The WatchDescriptor can be used to remove the watch using Watches::remove,
    /// thereby preventing future events of this type from being created.
    pub descriptor: i32,
    /// Indicates what kind of event this is
    pub mask: WatcherEvents,
    /// Connects related events to each other
    /// When a file is renamed, this results two events: MOVED_FROM and MOVED_TO. The cookie field will be the same for both of them, thereby making is possible to connect the event pair.
    pub cookie: u32,
    /// The name of the file the event originates from
    /// This field is set only if the subject of the event is a file or directory in a watched directory.
    /// If the event concerns a file or directory that is watched directly, name will be None.
    pub name: Option<String>,
}

impl From<inotify::Event<&OsStr>> for WatcherOutcome {
    fn from(event: inotify::Event<&OsStr>) -> Self {
        let name = event
            .name
            .map(|inner_name| inner_name.to_string_lossy().to_string());

        Self {
            descriptor: event.wd.get_watch_descriptor_id(),
            mask: event.mask.into(),
            cookie: event.cookie,
            name,
        }
    }
}
