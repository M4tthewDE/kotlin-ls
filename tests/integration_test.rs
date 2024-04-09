extern crate kotlin_ls;

use kotlin_ls::kotlin;

#[test]
fn test_dankchat() {
    for (path, file) in kotlin::from_path("DankChat").unwrap() {
        match file {
            Ok(f) => {
                if path.file_name().unwrap().to_str().unwrap() == "DankChatApplication.kt" {
                    dbg!(f);
                }
            }
            Err(err) => panic!("{:?}", err),
        }
    }
}
