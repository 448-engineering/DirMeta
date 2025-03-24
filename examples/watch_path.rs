#[cfg(all(feature = "async", feature = "watcher"))]
use dir_meta::{async_channel::unbounded, inotify::WatchMask, FsWatcher, WatcherOutcome};

fn main() {
    #[cfg(all(feature = "async", feature = "watcher"))]
    smol::block_on(async {
        let (sender, receiver) = unbounded::<WatcherOutcome>();

        let watch_options =
            WatchMask::MODIFY | WatchMask::CREATE | WatchMask::DELETE | WatchMask::DELETE_SELF;

        smol::spawn(async move {
            FsWatcher::new(sender)
                .path("src")
                .watch(watch_options)
                .await
                .unwrap();
        })
        .detach();

        while let Ok(data) = receiver.recv().await {
            dbg!(data);
        }
    });
}
