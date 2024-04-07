extern crate kotlin_ls;

use kotlin_ls::kotlin;

#[test]
fn test_dankchat() {
    tracing_subscriber::fmt().init();
    kotlin::from_path("DankChat").unwrap();
}
