use anyhow::{bail, Result};
use tree_sitter::Node;

use super::{class::ClassBody, delegation::Delegation};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Literal {
    Boolean(String),
    String(String),
    Integer(String),
    Object(ClassBody, Vec<Delegation>),
    Null,
}

impl Literal {
    pub fn new(node: &Node, content: &[u8]) -> Result<Literal> {
        match node.kind() {
            "boolean_literal" => Ok(Literal::Boolean(node.utf8_text(content)?.to_string())),
            "string_literal" => Ok(Literal::String(node.utf8_text(content)?.to_string())),
            "integer_literal" => Ok(Literal::Integer(node.utf8_text(content)?.to_string())),
            "object_literal" => {
                let mut delegations = Vec::new();
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    match child.kind() {
                        "object" | ":" => {}
                        "delegation_specifier" => {
                            delegations.push(Delegation::new(&child, content)?)
                        }
                        "class_body" => {
                            return Ok(Literal::Object(
                                ClassBody::new_class_body(&child, content)?,
                                delegations,
                            ))
                        }
                        _ => {
                            bail!(
                                "[Literal] unhandled node {} '{}' at {}",
                                child.kind(),
                                child.utf8_text(content)?,
                                child.start_position(),
                            )
                        }
                    }
                }

                bail!("[Literal] no class_body at {}", node.start_position());
            }
            "null" => Ok(Literal::Null),
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
