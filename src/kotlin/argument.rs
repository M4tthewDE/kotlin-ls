use anyhow::{bail, Context, Result};
use tree_sitter::Node;

use crate::kotlin::expression::Expression;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct ValueArgument {
    expression: Expression,
}

impl ValueArgument {
    fn new(node: &Node, content: &[u8]) -> Result<ValueArgument> {
        let mut expression = None;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "call_expression" | "navigation_expression" => {
                    expression = Some(Expression::new(&child, content)?)
                }
                _ => {
                    bail!(
                        "unhandled child {} '{}' at {}",
                        child.kind(),
                        child.utf8_text(content)?,
                        child.start_position(),
                    )
                }
            }
        }

        Ok(ValueArgument {
            expression: expression.context("no expression found")?,
        })
    }
}

pub fn get_arguments(node: &Node, content: &[u8]) -> Result<Vec<ValueArgument>> {
    let mut arguments = Vec::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "(" | ")" => {}
            "value_argument" => arguments.push(ValueArgument::new(&child, content)?),
            _ => {
                bail!(
                    "unhandled child {} '{}' at {}",
                    child.kind(),
                    child.utf8_text(content)?,
                    child.start_position(),
                )
            }
        }
    }

    Ok(arguments)
}
