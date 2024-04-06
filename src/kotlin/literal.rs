use anyhow::{bail, Result};
use tree_sitter::Node;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Literal {
    Boolean(String),
    Null,
}

impl Literal {
    pub fn new(node: &Node, content: &[u8]) -> Result<Literal> {
        match node.kind() {
            "boolean_literal" => Ok(Literal::Boolean(node.utf8_text(content)?.to_string())),
            _ => {
                bail!(
                    "[Literal] unhandled node {} '{}' at {}",
                    node.kind(),
                    node.utf8_text(content)?,
                    node.start_position(),
                )
            }
        }
    }
}
