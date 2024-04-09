extern crate kotlin_ls;

use kotlin_ls::kotlin;
use tracing::{debug, error};

#[test]
fn test_dankchat() {
    tracing_subscriber::fmt().init();

    let mut failures = Vec::new();

    for (path, file) in kotlin::from_path("DankChat").unwrap() {
        match file {
            Ok(f) => {
                if path.file_name().unwrap().to_str().unwrap() == "DankChatApplication.kt" {
                    debug!("{:?}", f);
                }
            }
            Err(err) => {
                failures.push(err);
            }
        }
    }

    if !failures.is_empty() {
        for failure in failures {
            error!("{failure:?}")
        }

        panic!("There were failures!");
    }
}
