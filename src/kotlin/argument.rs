use anyhow::{bail, Result};
use tree_sitter::Node;

use crate::kotlin::expression::Expression;

use super::literal::Literal;

// TODO: this works better as an enum
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct ValueArgument {
    expression: Option<Expression>,
    identifier: Option<String>,
    literal: Option<Literal>,
}

impl ValueArgument {
    fn new(node: &Node, content: &[u8]) -> Result<ValueArgument> {
        let mut expression = None;
        let mut identifier = None;
        let mut literal = None;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "=" => {}
                "boolean_literal" => literal = Some(Literal::new(&child, content)?),
                "call_expression" | "navigation_expression" => {
                    expression = Some(Expression::new(&child, content)?)
                }
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

        Ok(ValueArgument {
            expression,
            identifier,
            literal,
        })
    }
}

pub fn get_arguments(node: &Node, content: &[u8]) -> Result<Vec<ValueArgument>> {
    let mut arguments = Vec::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "(" | ")" | "," => {}
            "value_argument" => arguments.push(ValueArgument::new(&child, content)?),
            _ => {
                bail!(
                    "[get_arguments] unhandled child {} '{}' at {}",
                    child.kind(),
                    child.utf8_text(content)?,
                    child.start_position(),
                )
            }
        }
    }

    Ok(arguments)
}
