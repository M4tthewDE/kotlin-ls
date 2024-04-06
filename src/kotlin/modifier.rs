use anyhow::{bail, Result};
use tree_sitter::Node;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Modifier {
    Class(String),
    Visibility(String),
    Annotation(String),
    Inheritance(String),
    Member(String),
}

impl Modifier {
    pub fn new(node: &Node, content: &[u8]) -> Result<Modifier> {
        match node.kind() {
            "visibility_modifier" => Ok(Modifier::Visibility(node.utf8_text(content)?.to_string())),
            "class_modifier" => Ok(Modifier::Class(node.utf8_text(content)?.to_string())),
            "annotation" => Ok(Modifier::Annotation(node.utf8_text(content)?.to_string())),
            "inheritance_modifier" => {
                Ok(Modifier::Inheritance(node.utf8_text(content)?.to_string()))
            }
            "member_modifier" => Ok(Modifier::Member(node.utf8_text(content)?.to_string())),
            _ => bail!("unknown modifier {}", node.kind()),
        }
    }
}
