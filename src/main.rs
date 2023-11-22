mod utils;
pub use utils::*;

mod fs;
pub use fs::*;

fn main() {
    smol::block_on(async {
        let outcome = DirMetadata::new("src").dir_metadata().await.unwrap();

        dbg!(&outcome);
        dbg!(outcome.size_formatted());

        for file in outcome.files() {
            dbg!(file.name());
            dbg!(file.accessed_24hr());
            dbg!(file.accessed_am_pm());
            dbg!(file.accessed_humatime());
            dbg!(file.created_24hr());
            dbg!(file.created_am_pm());
            dbg!(file.created_humatime());
            dbg!(file.modified_24hr());
            dbg!(file.modified_am_pm());
            dbg!(file.modified_humatime());
            dbg!(file.formatted_size());
        }
    })
}
