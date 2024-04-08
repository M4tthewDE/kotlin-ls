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
        let mut data_type = None;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "user_type" => data_type = Some(Type::new(&child, content)?),
                _ => {
                    bail!(
                        "[TypeProjection] unhandled child {} '{}' at {}",
                        child.kind(),
                        child.utf8_text(content)?,
                        child.start_position(),
                    )
                }
            }
        }

        Ok(TypeProjection {
            data_type: data_type.context(format!(
                "[TypeProjection] no data type found at {}",
                node.start_position()
            ))?,
        })
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Argument {
    Value {
        expression: Expression,
        identifier: Option<String>,
    },
    Type {
        type_projections: Vec<TypeProjection>,
    },
}

impl Argument {
    fn new_value_argument(node: &Node, content: &[u8]) -> Result<Argument> {
        let mut expression = None;
        let mut identifier = None;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "=" => {}
                "call_expression"
                | "navigation_expression"
                | "infix_expression"
                | "as_expression"
                | "boolean_literal"
                | "null"
                | "string_literal"
                | "lambda_literal"
                | "integer_literal"
                | "elvis_expression"
                | "equality_expression"
                | "callable_reference" => expression = Some(Expression::new(&child, content)?),
                "simple_identifier" => {
                    // simple_identifier has to be followed by "=", else it's an expression
                    if child.next_sibling().is_some() {
                        identifier = Some(child.utf8_text(content)?.to_string());
                    } else {
                        expression = Some(Expression::new(&child, content)?)
                    }
                }
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
            expression: expression.context(format!(
                "[ValueArgument] no expression found at {}",
                node.start_position()
            ))?,
            identifier,
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
