use anyhow::{bail, Context, Result};
use tree_sitter::Node;

use super::{constructor_invocation::ConstructorInvocation, types::Type};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Delegation {
    Type(Type),
    ConstructorInvocation(ConstructorInvocation),
}

impl Delegation {
    pub fn new(node: &Node, content: &[u8]) -> Result<Delegation> {
        let child = node.child(0).context("no delegation specifier child")?;
        match child.kind() {
            "user_type" => Ok(Delegation::Type(Type::new(&child, content)?)),
            "constructor_invocation" => Ok(Delegation::ConstructorInvocation(
                ConstructorInvocation::new(&child, content)?,
            )),
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
}
