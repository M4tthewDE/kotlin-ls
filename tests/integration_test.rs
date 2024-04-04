extern crate kotlin_ls;

use kotlin_ls::kotlin;

#[test]
fn test_dankchat() {
    kotlin::from_path("DankChat").unwrap();
}
