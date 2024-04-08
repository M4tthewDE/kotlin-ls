use anyhow::{bail, Context, Result};
use tree_sitter::Node;

use super::types::Type;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct VariableDeclaration {
    identifier: String,
    data_type: Option<Type>,
}

impl VariableDeclaration {
    pub fn new(node: &Node, content: &[u8]) -> Result<VariableDeclaration> {
        let mut identifier = None;
        let mut data_type = None;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                ":" => {}
                "simple_identifier" => identifier = Some(child.utf8_text(content)?.to_string()),
                "user_type" | "nullable_type" => data_type = Some(Type::new(&child, content)?),
                _ => {
                    bail!(
                        "[VariableDeclaration] unhandled child {} '{}' at {}",
                        child.kind(),
                        child.utf8_text(content)?,
                        child.start_position(),
                    )
                }
            }
        }

        Ok(VariableDeclaration {
            identifier: identifier.context("no identifier found")?,
            data_type,
        })
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct MultiVariableDeclaration {
    variable_declarations: Vec<VariableDeclaration>,
}

impl MultiVariableDeclaration {
    pub fn new(node: &Node, content: &[u8]) -> Result<MultiVariableDeclaration> {
        let mut vars = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "(" | "," | ")" => {}
                "variable_declaration" => vars.push(VariableDeclaration::new(&child, content)?),
                _ => {
                    bail!(
                        "[MultiVariableDeclaration] unhandled child {} '{}' at {}",
                        child.kind(),
                        child.utf8_text(content)?,
                        child.start_position(),
                    )
                }
            }
        }

        Ok(MultiVariableDeclaration {
            variable_declarations: vars,
        })
    }
}
