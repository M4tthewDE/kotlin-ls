use anyhow::{bail, Context, Result};
use tree_sitter::Node;

use super::expression::{Expression, EXPRESSIONS};

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
        let mut left = None;
        let mut right = None;
        let mut operator = None;
        let mut cursor = node.walk();

        for child in node.children(&mut cursor.clone()) {
            if child.kind() == "directly_assignable_expression" {
                for child in child.children(&mut cursor) {
                    if EXPRESSIONS.contains(&child.kind()) {
                        left = Some(Expression::new(&child, content)?);
                    }
                }
            } else if EXPRESSIONS.contains(&child.kind()) {
                right = Some(Expression::new(&child, content)?);
            } else if let Ok(op) = AssignmentOperator::new(&child) {
                operator = Some(op);
            }
        }

        Ok(Assignment {
            left: left.context(format!(
                "[Assignment] no left expression found at {} - {}",
                node.start_position(),
                node.end_position(),
            ))?,
            operator: operator.context(format!(
                "[Assignment] no operator found at {} - {}",
                node.start_position(),
                node.end_position(),
            ))?,
            right: right.context(format!(
                "[Assignment] no right expression found at {} - {}",
                node.start_position(),
                node.end_position(),
            ))?,
        })
    }
}
