use anyhow::{bail, Context, Result};
use tree_sitter::Node;

use crate::kotlin::{
    statement::{self, Statement},
    types::Type,
};

use super::Expression;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct CatchBlock {
    pub identifier: String,
    pub typ: Type,
    pub block: Vec<Statement>,
}

pub fn expression(node: &Node, content: &[u8]) -> Result<Expression> {
    let block = statement::get_statements(
        &node.child(2).context(format!(
            "[Expression::Try] no child at {}",
            node.start_position()
        ))?,
        content,
    )?;

    let mut catch_blocks = Vec::new();
    let mut finally_block = None;
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "try" | "{" | "}" | "statements" => {}
            "catch_block" => catch_blocks.push(CatchBlock::new(&child, content)?),
            "finally_block" => finally_block = Some(FinallyBlock::new(&child, content)?),
            _ => {
                bail!(
                    "[Expression::Try] unhandled child {} '{}' at {}",
                    child.kind(),
                    child.utf8_text(content)?,
                    child.start_position(),
                )
            }
        }
    }

    Ok(Expression::Try {
        block,
        catch_blocks,
        finally_block,
    })
}

impl CatchBlock {
    fn new(node: &Node, content: &[u8]) -> Result<CatchBlock> {
        let statements_node = node.child(node.child_count() - 2).context(format!(
            "[CatchBlock] no child at {}",
            node.start_position()
        ))?;

        match statements_node.kind() {
            "statements" => Ok(CatchBlock {
                identifier: node
                    .child(node.child_count() - 7)
                    .context(format!(
                        "[CatchBlock] no child at {}",
                        node.start_position()
                    ))?
                    .utf8_text(content)?
                    .to_string(),
                typ: Type::new(
                    &node.child(node.child_count() - 5).context(format!(
                        "[CatchBlock] no child at {}",
                        node.start_position()
                    ))?,
                    content,
                )?,
                block: statement::get_statements(
                    &node.child(node.child_count() - 2).context(format!(
                        "[CatchBlock] no child at {}",
                        node.start_position()
                    ))?,
                    content,
                )?,
            }),
            _ => Ok(CatchBlock {
                identifier: node
                    .child(node.child_count() - 6)
                    .context(format!(
                        "[CatchBlock] no child at {}",
                        node.start_position()
                    ))?
                    .utf8_text(content)?
                    .to_string(),
                typ: Type::new(
                    &node.child(node.child_count() - 4).context(format!(
                        "[CatchBlock] no child at {}",
                        node.start_position()
                    ))?,
                    content,
                )?,
                block: Vec::new(),
            }),
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct FinallyBlock {
    pub block: Vec<Statement>,
}

impl FinallyBlock {
    fn new(node: &Node, content: &[u8]) -> Result<FinallyBlock> {
        Ok(FinallyBlock {
            block: statement::get_statements(
                &node.child(0).context(format!(
                    "[FinallyBlock] no child at {}",
                    node.start_position()
                ))?,
                content,
            )?,
        })
    }
}
