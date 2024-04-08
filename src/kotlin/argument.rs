use anyhow::{bail, Context, Result};
use tree_sitter::Node;

use crate::kotlin::expression::Expression;

use super::types::Type;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct TypeProjection {
    data_type: Type,
}

impl TypeProjection {
    fn new(node: &Node, content: &[u8]) -> Result<TypeProjection> {
        Ok(TypeProjection {
            data_type: Type::new(
                &node.child(node.child_count() - 1).context(format!(
                    "[TypeProjection] no child at {}",
                    node.start_position()
                ))?,
                content,
            )?,
        })
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Argument {
    Value {
        annotation: Option<String>,
        identifier: Option<String>,
        expression: Expression,
    },
    Type {
        type_projections: Vec<TypeProjection>,
    },
}

impl Argument {
    fn new_value_argument(node: &Node, content: &[u8]) -> Result<Argument> {
        let mut identifier = None;
        let mut annotation = None;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor).take(node.child_count() - 1) {
            match child.kind() {
                "=" => {}
                "annotation" => annotation = Some(child.utf8_text(content)?.to_string()),
                "simple_identifier" => identifier = Some(child.utf8_text(content)?.to_string()),
                _ => {
                    bail!(
                        "[ValueArgument] unhandled child {} '{}' at {}",
                        child.kind(),
                        child.utf8_text(content)?,
                        child.start_position(),
                    )
                }
            }
        }

        Ok(Argument::Value {
            annotation,
            identifier,
            expression: Expression::new(
                &node.child(node.child_count() - 1).context(format!(
                    "[ValueArgument] no child at {}",
                    node.start_position()
                ))?,
                content,
            )?,
        })
    }
}

pub fn get_value_arguments(node: &Node, content: &[u8]) -> Result<Vec<Argument>> {
    let mut arguments = Vec::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "(" | ")" | "," => {}
            "value_argument" => arguments.push(Argument::new_value_argument(&child, content)?),
            _ => {
                bail!(
                    "[get_value_arguments] unhandled child {} '{}' at {}",
                    child.kind(),
                    child.utf8_text(content)?,
                    child.start_position(),
                )
            }
        }
    }

    Ok(arguments)
}

pub fn get_type_argument(node: &Node, content: &[u8]) -> Result<Argument> {
    let mut type_projections = Vec::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "<" | ">" | "," => {}
            "type_projection" => type_projections.push(TypeProjection::new(&child, content)?),
            _ => {
                bail!(
                    "[get_type_argument] unhandled child {} '{}' at {}",
                    child.kind(),
                    child.utf8_text(content)?,
                    child.start_position(),
                )
            }
        }
    }

    Ok(Argument::Type { type_projections })
}
