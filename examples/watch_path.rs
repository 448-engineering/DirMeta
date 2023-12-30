use dir_meta::{inotify::WatchMask, smol::channel, FsWatcher, WatcherOutcome};

fn main() {
    smol::block_on(async {
        let (sender, receiver) = channel::unbounded::<WatcherOutcome>();

        let watch_options =
            WatchMask::MODIFY | WatchMask::CREATE | WatchMask::DELETE | WatchMask::DELETE_SELF;

        smol::spawn(FsWatcher::new(sender).path("Foo").watch(watch_options)).detach();

        while let Ok(data) = receiver.recv().await {
            dbg!(data);
        }
    });
}
