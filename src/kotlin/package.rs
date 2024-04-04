use anyhow::{bail, Context, Result};
use tree_sitter::Tree;

#[derive(Debug)]
pub struct Package(String);

pub fn get_package(tree: &Tree, content: &[u8]) -> Result<Package> {
    let mut cursor = tree.walk();
    loop {
        let node = cursor.node();
        if node.kind() == "package" {
            let package = node
                .next_sibling()
                .context("no package found")?
                .utf8_text(content)?
                .to_string();

            return Ok(Package(package));
        }

        if cursor.goto_first_child() {
            continue;
        }

        loop {
            if cursor.goto_next_sibling() {
                break;
            }

            if !cursor.goto_parent() {
                bail!("no package found");
            }
        }
    }
}
