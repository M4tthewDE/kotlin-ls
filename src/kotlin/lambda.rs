use anyhow::{bail, Context, Result};
use tree_sitter::Node;

use super::literal::Literal;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct AnnotatedLambda {
    lambda_literal: Literal,
}

impl AnnotatedLambda {
    pub fn new(node: &Node, content: &[u8]) -> Result<AnnotatedLambda> {
        let mut lambda_literal = None;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "lambda_literal" => lambda_literal = Some(Literal::new(&child, content)?),
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
