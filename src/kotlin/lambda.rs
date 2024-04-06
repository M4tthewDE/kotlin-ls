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
}

impl LambdaLiteral {
    pub fn new(node: &Node, content: &[u8]) -> Result<LambdaLiteral> {
        let mut statements = None;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "{" => {}
                "statements" => statements = Some(statement::get_statements(&child, content)?),
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

        Ok(LambdaLiteral { statements })
    }
}
