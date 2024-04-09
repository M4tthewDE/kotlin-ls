use crate::kotlin::property::Property;
use anyhow::{bail, Result};
use tree_sitter::Node;

use super::{
    assignment::Assignment,
    expression::{Expression, EXPRESSIONS},
};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Statement {
    PropertyDeclaration(Property),
    Expression(Expression),
    Assignment(Assignment),
}

pub fn get_statements(node: &Node, content: &[u8]) -> Result<Vec<Statement>> {
    let mut statements = Vec::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "line_comment" => {}
            "property_declaration" => statements.push(Statement::PropertyDeclaration(
                Property::new(&child, content)?,
            )),
            "assignment" => {
                statements.push(Statement::Assignment(Assignment::new(&child, content)?))
            }
            kind => {
                if EXPRESSIONS.contains(&kind) {
                    statements.push(Statement::Expression(Expression::new(&child, content)?))
                } else {
                    bail!(
                        "[get_statementes] unhandled child {} '{}' at {}",
                        child.kind(),
                        child.utf8_text(content)?,
                        child.start_position(),
                    )
                }
            }
        }
    }

    Ok(statements)
}
