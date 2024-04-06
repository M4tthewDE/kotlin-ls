use anyhow::{bail, Context, Result};
use tree_sitter::Node;

use super::argument::{self, ValueArgument};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Expression {
    Call {
        identifier: Option<String>,
        call_suffix: CallSuffix,
        expression: Box<Option<Expression>>,
    },
    Navigation {
        identifier: Option<String>,
        navigation_suffix: NavigationSuffix,
        expression: Box<Option<Expression>>,
    },
}

impl Expression {
    pub fn new(node: &Node, content: &[u8]) -> Result<Expression> {
        match node.kind() {
            "call_expression" => call_expression(node, content),
            "navigation_expression" => navigation_expression(node, content),
            _ => {
                bail!(
                    "[Expression] unhandled child {} '{}' at {}",
                    node.kind(),
                    node.utf8_text(content)?,
                    node.start_position(),
                )
            }
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct CallSuffix {
    arguments: Vec<ValueArgument>,
}

impl CallSuffix {
    pub fn new(node: &Node, content: &[u8]) -> Result<CallSuffix> {
        let mut arguments = None;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "value_arguments" => arguments = Some(argument::get_arguments(&child, content)?),
                _ => {
                    bail!(
                        "[CallSuffix] unhandled child {} '{}' at {}",
                        child.kind(),
                        child.utf8_text(content)?,
                        child.start_position(),
                    )
                }
            }
        }

        Ok(CallSuffix {
            arguments: arguments.context("no arguments found")?,
        })
    }
}

fn call_expression(node: &Node, content: &[u8]) -> Result<Expression> {
    let mut identifier = None;
    let mut call_suffix = None;
    let mut expression = None;
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "simple_identifier" => identifier = Some(child.utf8_text(content)?.to_string()),
            "call_suffix" => call_suffix = Some(CallSuffix::new(&child, content)?),
            "navigation_expression" => expression = Some(Expression::new(&child, content)?),
            _ => {
                bail!(
                    "[Expression::Call] unhandled child {} '{}' at {}",
                    child.kind(),
                    child.utf8_text(content)?,
                    child.start_position(),
                )
            }
        }
    }

    Ok(Expression::Call {
        identifier,
        call_suffix: call_suffix.context("no call suffix found")?,
        expression: Box::new(expression),
    })
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct NavigationSuffix {
    identifier: String,
}

impl NavigationSuffix {
    pub fn new(node: &Node, content: &[u8]) -> Result<NavigationSuffix> {
        let mut identifier = None;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "." => {}
                "simple_identifier" => identifier = Some(child.utf8_text(content)?.to_string()),
                _ => {
                    bail!(
                        "[NavigationSuffix] unhandled child {} '{}' at {}",
                        child.kind(),
                        child.utf8_text(content)?,
                        child.start_position(),
                    )
                }
            }
        }

        Ok(NavigationSuffix {
            identifier: identifier.context("no identifier found")?,
        })
    }
}

fn navigation_expression(node: &Node, content: &[u8]) -> Result<Expression> {
    let mut identifier = None;
    let mut navigation_suffix = None;
    let mut expression = None;
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "simple_identifier" => identifier = Some(child.utf8_text(content)?.to_string()),
            "call_expression" | "navigation_expression" => {
                expression = Some(Expression::new(&child, content)?)
            }
            "navigation_suffix" => {
                navigation_suffix = Some(NavigationSuffix::new(&child, content)?)
            }
            _ => {
                bail!(
                    "[Expression::Navigation] unhandled child {} '{}' at {}",
                    child.kind(),
                    child.utf8_text(content)?,
                    child.start_position(),
                )
            }
        }
    }

    Ok(Expression::Navigation {
        identifier,
        navigation_suffix: navigation_suffix.context("no call suffix found")?,
        expression: Box::new(expression),
    })
}
