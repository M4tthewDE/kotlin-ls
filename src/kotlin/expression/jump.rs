use anyhow::{bail, Context, Result};
use tree_sitter::Node;

use crate::kotlin::label::Label;

use super::Expression;

pub fn expression(node: &Node, content: &[u8]) -> Result<Expression> {
    Ok(
        match node
            .child(0)
            .context(format!(
                "[Expression::Jump] no child at {}",
                node.start_position()
            ))?
            .kind()
        {
            "throw" => Expression::JumpThrow(Box::new(Expression::new(
                &node.child(1).context(format!(
                    "[Expression::Jump] no child at {}",
                    node.start_position()
                ))?,
                content,
            )?)),
            "return" => Expression::JumpReturn(None, None),
            "return@" => Expression::JumpReturn(
                Some(Label::new(
                    &node.child(1).context(format!(
                        "[Expression::Jump] no child at {}",
                        node.start_position()
                    ))?,
                    content,
                )?),
                if let Some(child) = &node.child(2) {
                    Some(Box::new(Expression::new(child, content)?))
                } else {
                    None
                },
            ),
            "continue" => Expression::JumpContinue(None),
            "continue@" => Expression::JumpContinue(Some(Label::new(
                &node.child(1).context(format!(
                    "[Expression::Jump] no child at {}",
                    node.start_position()
                ))?,
                content,
            )?)),
            "break" => Expression::JumpBreak(None),
            "break@" => Expression::JumpBreak(Some(Label::new(
                &node.child(1).context(format!(
                    "[Expression::Jump] no child at {}",
                    node.start_position()
                ))?,
                content,
            )?)),
            jump => {
                bail!(
                    "[Expression::Jump] unhandled jump {} at {}",
                    jump,
                    node.start_position(),
                )
            }
        },
    )
}
