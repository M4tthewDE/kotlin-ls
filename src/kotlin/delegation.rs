use anyhow::{bail, Result};
use tree_sitter::Node;

use super::{constructor_invocation::ConstructorInvocation, types::Type};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Delegation {
    Type(Type),
    ConstructorInvocation(ConstructorInvocation),
}

impl Delegation {
    pub fn new(node: &Node, content: &[u8]) -> Result<Delegation> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "line_comment" | "multi_line_comment" => {}
                "user_type" => return Ok(Delegation::Type(Type::new(&child, content)?)),
                "constructor_invocation" => {
                    return Ok(Delegation::ConstructorInvocation(
                        ConstructorInvocation::new(&child, content)?,
                    ))
                }
                _ => {
                    bail!(
                        "[Delegation] unhandled child {} '{}' at {}",
                        child.kind(),
                        child.utf8_text(content)?,
                        child.start_position(),
                    )
                }
            }
        }
        bail!(
            "[Delegation] no child at {} - {}",
            node.start_position(),
            node.end_position(),
        )
    }
}
