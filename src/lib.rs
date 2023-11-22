#![deny(unsafe_code)]
#![forbid(missing_docs)]
#![doc = include_str!("../README.md")]

mod utils;
pub use utils::*;

mod fs;
pub use fs::*;

#[cfg(test)]
mod sanity_checks {
    #[test]
    fn ineq() {
        smol::block_on(async {
            let outcome = crate::DirMetadata::new("src").dir_metadata().await.unwrap();

            dbg!(&outcome);
            dbg!(outcome.size_formatted());

            {
                #[cfg(feature = "time")]
                for file in outcome.files() {
                    assert_ne!("", file.name());
                    assert_ne!(Option::None, file.accessed_24hr());
                    assert_ne!(Option::None, file.accessed_am_pm());
                    assert_ne!(Option::None, file.accessed_humatime());
                    assert_ne!(Option::None, file.created_24hr());
                    assert_ne!(Option::None, file.created_am_pm());
                    assert_ne!(Option::None, file.created_humatime());
                    assert_ne!(Option::None, file.modified_24hr());
                    assert_ne!(Option::None, file.modified_am_pm());
                    assert_ne!(Option::None, file.modified_humatime());
                    assert_ne!(String::default(), file.formatted_size());
                }
            }
        })
    }
}
