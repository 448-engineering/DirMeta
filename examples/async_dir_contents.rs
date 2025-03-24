fn main() {
    #[cfg(all(feature = "async", feature = "size"))]
    smol::block_on(async move {
        let dir = String::from(env!("CARGO_MANIFEST_DIR")) + "/src";
        dbg!(&dir);

        let outcome = dir_meta::DirMetadata::new(&dir).async_dir_metadata().await;
        dbg!(&outcome);
        dbg!(&outcome.unwrap().size_formatted());
    });
}
