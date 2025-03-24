fn main() {
    #[cfg(all(feature = "sync", feature = "size"))]
    {
        let dir = String::from(env!("CARGO_MANIFEST_DIR")) + "/src";

        let outcome = dir_meta::DirMetadata::new(&dir).sync_dir_metadata();
        dbg!(&outcome);
        dbg!(&outcome.unwrap().size_formatted());
    }
}
