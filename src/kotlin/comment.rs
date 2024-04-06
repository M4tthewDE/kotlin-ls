use anyhow::{bail, Result};
use tree_sitter::Node;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Comment {
    Multiline(String),
}

impl Comment {
    pub fn new(node: &Node, content: &[u8]) -> Result<Comment> {
        match node.kind() {
            "multiline_comment" => Ok(Comment::Multiline(node.utf8_text(content)?.to_string())),
            _ => {
                bail!(
                    "[Comment] unhandled node {} '{}' at {}",
                    node.kind(),
                    node.utf8_text(content)?,
                    node.start_position(),
                )
            }
        }
    }
}
