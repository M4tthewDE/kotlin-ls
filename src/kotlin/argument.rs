use anyhow::{bail, Context, Result};
use tree_sitter::Node;

use crate::kotlin::expression::Expression;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct ValueArgument {
    expression: Expression,
    identifier: Option<String>,
}

impl ValueArgument {
    fn new(node: &Node, content: &[u8]) -> Result<ValueArgument> {
        let mut expression = None;
        let mut identifier = None;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "=" => {}
                "call_expression"
                | "navigation_expression"
                | "infix_expression"
                | "boolean_literal"
                | "null"
                | "string_literal" => expression = Some(Expression::new(&child, content)?),
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

        Ok(ValueArgument {
            expression: expression.context(format!(
                "[ValueArgument] no expression found at {}",
                node.start_position()
            ))?,
            identifier,
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
