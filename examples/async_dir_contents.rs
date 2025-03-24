fn main() {
    #[cfg(all(feature = "async", feature = "size"))]
    smol::block_on(async move {
        // let dir = String::from(env!("CARGO_MANIFEST_DIR")) + "/src";
        let dir = String::from(env!("CARGO_MANIFEST_DIR")) + "/src";
        dbg!(&dir);

        let outcome = dir_meta::DirMetadata::new(&dir).async_dir_metadata().await;
        dbg!(&outcome);
        dbg!(&outcome.as_ref().unwrap().size_formatted());

        let path = dir.clone() + "/lib.rs";
        dbg!(&path);

        let file = outcome.as_ref().unwrap().get_file_by_path(&path);
        assert!(file.is_some());
    });
}
