extern crate kotlin_ls;

use kotlin_ls::kotlin;
use tracing::error;

#[test]
fn test_dankchat() {
    tracing_subscriber::fmt().init();
    if let Err(err) = kotlin::from_path("DankChat") {
        error!("{err:?}");
        panic!("failed to parse DankChat");
    }
}
