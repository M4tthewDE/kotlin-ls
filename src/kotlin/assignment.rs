use anyhow::{bail, Context, Result};
use tree_sitter::Node;

use super::expression::Expression;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum AssignmentOperator {
    Equals,
    Plus,
    Minus,
    Mul,
    Div,
    Mod,
}
impl AssignmentOperator {
    pub fn new(node: &Node) -> Result<AssignmentOperator> {
        Ok(match node.kind() {
            "=" => AssignmentOperator::Equals,
            "+=" => AssignmentOperator::Plus,
            "-=" => AssignmentOperator::Minus,
            "*=" => AssignmentOperator::Mul,
            "/=" => AssignmentOperator::Div,
            "%=" => AssignmentOperator::Mod,
            operator => bail!(
                "[AssignmentOperator] unknown operator {operator} at {}",
                node.start_position()
            ),
        })
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Assignment {
    operator: AssignmentOperator,
    left: Expression,
    right: Expression,
}

impl Assignment {
    pub fn new(node: &Node, content: &[u8]) -> Result<Assignment> {
        Ok(Assignment {
            left: Expression::new(
                &node.child(0).context(format!(
                    "[Assignment] no expression found at {}",
                    node.start_position()
                ))?,
                content,
            )?,
            operator: AssignmentOperator::new(&node.child(1).context(format!(
                "[Assignment] no operator found at {}",
                node.start_position()
            ))?)?,
            right: Expression::new(
                &node.child(2).context(format!(
                    "[Assignment] no expression found at {}",
                    node.start_position()
                ))?,
                content,
            )?,
        })
    }
}
