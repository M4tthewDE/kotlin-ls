use crate::kotlin::property::Property;
use anyhow::{bail, Context, Result};
use tree_sitter::Node;

use super::{
    assignment::Assignment,
    expression::{ControlStructureBody, Expression, EXPRESSIONS},
    function::Function,
};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Statement {
    PropertyDeclaration(Property),
    Expression(Expression),
    Assignment(Assignment),
    Function(Function),
    While(Expression, Option<ControlStructureBody>),
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
            "function_declaration" => {
                statements.push(Statement::Function(Function::new(&child, content)?))
            }
            "assignment" => {
                statements.push(Statement::Assignment(Assignment::new(&child, content)?))
            }
            "while_statement" => statements.push(while_statement(&child, content)?),
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

fn while_statement(node: &Node, content: &[u8]) -> Result<Statement> {
    if let Some(last) = node.child(node.child_count() - 1) {
        if last.kind() == ";" {
            Ok(Statement::While(
                Expression::new(
                    &node.child(2).context(format!(
                        "[Statement::While] no child at {}",
                        node.start_position()
                    ))?,
                    content,
                )?,
                None,
            ))
        } else {
            Ok(Statement::While(
                Expression::new(
                    &node.child(2).context(format!(
                        "[Statement::While] no child at {}",
                        node.start_position()
                    ))?,
                    content,
                )?,
                ControlStructureBody::new(&last, content).ok(),
            ))
        }
    } else {
        bail!("[Statement::While] no child at {}", node.start_position());
    }
}
