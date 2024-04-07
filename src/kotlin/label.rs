use anyhow::Result;
use tree_sitter::Node;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Label {
    label: String,
}

impl Label {
    pub fn new(node: &Node, content: &[u8]) -> Result<Label> {
        Ok(Label {
            label: node.utf8_text(content)?.to_string(),
        })
    }
}
