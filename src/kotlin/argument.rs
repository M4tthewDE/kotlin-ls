use anyhow::{bail, Result};
use tree_sitter::Node;

use crate::kotlin::expression::{Expression, EXPRESSIONS};

use super::types::{Type, TYPES};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct TypeProjection {
    data_type: Type,
}

impl TypeProjection {
    fn new(node: &Node, content: &[u8]) -> Result<TypeProjection> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if TYPES.contains(&child.kind()) {
                return Ok(TypeProjection {
                    data_type: Type::new(&child, content)?,
                });
            }
        }
        bail!(
            "[TypeProjection] no type at {} - {}",
            node.start_position(),
            node.end_position(),
        )
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
        for child in node.children(&mut cursor) {
            let kind = child.kind();

            if kind == "annotation" {
                annotation = Some(child.utf8_text(content)?.to_string());
            }

            if (kind == "simple_identifier" && identifier.is_some()) || EXPRESSIONS.contains(&kind)
            {
                return Ok(Argument::Value {
                    annotation,
                    identifier,
                    expression: Expression::new(&child, content)?,
                });
            }

            if kind == "simple_identifier" {
                identifier = Some(child.utf8_text(content)?.to_string());
            }
        }
        bail!(
            "[ValueArgument] no expression at {} - {}",
            node.start_position(),
            node.end_position(),
        )
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
