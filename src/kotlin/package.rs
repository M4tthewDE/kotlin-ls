use anyhow::{bail, Context, Result};
use tree_sitter::Tree;

pub fn get_package(tree: &Tree, content: &[u8]) -> Result<String> {
    let mut cursor = tree.walk();
    loop {
        let node = cursor.node();
        if node.kind() == "package" {
            return Ok(node
                .next_sibling()
                .context("no package found")?
                .utf8_text(content)?
                .to_string());
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
