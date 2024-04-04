use anyhow::{Context, Result};
use tree_sitter::Tree;

pub fn get_imports(tree: &Tree, content: &[u8]) -> Result<Vec<String>> {
    let mut imports = Vec::new();
    let mut cursor = tree.walk();
    loop {
        let node = cursor.node();
        if node.kind() == "import" {
            let import = node
                .next_sibling()
                .context("malformed import")?
                .utf8_text(content)
                .context("malformed import")?
                .to_string();

            imports.push(import);
        }

        if cursor.goto_first_child() {
            continue;
        }

        loop {
            if cursor.goto_next_sibling() {
                break;
            }

            if !cursor.goto_parent() {
                return Ok(imports);
            }
        }
    }
}
