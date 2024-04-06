use anyhow::{bail, Context, Result};
use tree_sitter::Node;

use super::statement::{self, Statement};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct AnnotatedLambda {
    lambda_literal: LambdaLiteral,
}

impl AnnotatedLambda {
    pub fn new(node: &Node, content: &[u8]) -> Result<AnnotatedLambda> {
        let mut lambda_literal = None;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "lambda_literal" => lambda_literal = Some(LambdaLiteral::new(&child, content)?),
                _ => {
                    bail!(
                        "[AnnotatedLambda] unhandled child {} '{}' at {}",
                        child.kind(),
                        child.utf8_text(content)?,
                        child.start_position(),
                    )
                }
            }
        }

        Ok(AnnotatedLambda {
            lambda_literal: lambda_literal.context("no lambda_literal found")?,
        })
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct LambdaLiteral {
    statements: Option<Vec<Statement>>,
    parameters: Option<Vec<LambdaParameter>>,
}

impl LambdaLiteral {
    pub fn new(node: &Node, content: &[u8]) -> Result<LambdaLiteral> {
        let mut statements = None;
        let mut parameters = None;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "{" => {}
                "statements" => statements = Some(statement::get_statements(&child, content)?),
                "lambda_parameters" => parameters = Some(get_parameters(&child, content)?),
                _ => {
                    bail!(
                        "[LambdaLiteral] unhandled child {} '{}' at {}",
                        child.kind(),
                        child.utf8_text(content)?,
                        child.start_position(),
                    )
                }
            }
        }

        Ok(LambdaLiteral {
            statements,
            parameters,
        })
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct LambdaParameter {}

impl LambdaParameter {
    pub fn new(node: &Node, content: &[u8]) -> Result<LambdaParameter> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                _ => {
                    bail!(
                        "[LambdaParameter] unhandled child {} '{}' at {}",
                        child.kind(),
                        child.utf8_text(content)?,
                        child.start_position(),
                    )
                }
            }
        }

        Ok(LambdaParameter {})
    }
}

fn get_parameters(node: &Node, content: &[u8]) -> Result<Vec<LambdaParameter>> {
    let mut parameters = Vec::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            _ => {
                bail!(
                    "[get_parameters] unhandled child {} '{}' at {}",
                    child.kind(),
                    child.utf8_text(content)?,
                    child.start_position(),
                )
            }
        }
    }

    Ok(parameters)
}
