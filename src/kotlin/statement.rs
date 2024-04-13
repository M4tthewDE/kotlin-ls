use crate::kotlin::property::Property;
use anyhow::{bail, Context, Result};
use tree_sitter::Node;

use super::{
    assignment::Assignment,
    expression::{ControlStructureBody, Expression, EXPRESSIONS},
    function::Function,
    variable_declaration::{MultiVariableDeclaration, VariableDeclaration},
};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum ForParameter {
    VariableDeclaration(VariableDeclaration),
    MultiVariableDeclaration(MultiVariableDeclaration),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Statement {
    PropertyDeclaration(Property),
    Expression(Expression),
    Assignment(Assignment),
    Function(Function),
    While(Expression, Option<ControlStructureBody>),
    For(Expression, ForParameter, Option<ControlStructureBody>),
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
            "for_statement" => statements.push(for_statement(&child, content)?),
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
    let mut expression = None;
    let mut body = None;
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "control_structure_body" => {
                body = Some(ControlStructureBody::new(&child, content)?);
            }
            kind => {
                if EXPRESSIONS.contains(&kind) {
                    expression = Some(Expression::new(&child, content)?);
                }
            }
        }
    }

    Ok(Statement::While(
        expression.context(format!(
            "[Statement::While] no expression at {} - {}",
            node.start_position(),
            node.end_position()
        ))?,
        body,
    ))
}

fn for_statement(node: &Node, content: &[u8]) -> Result<Statement> {
    let mut parameter = None;
    let mut body = None;
    let mut expression = None;
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "variable_declaration" => {
                parameter = Some(ForParameter::VariableDeclaration(VariableDeclaration::new(
                    &child, content,
                )?));
            }
            "multi_variable_declaration" => {
                parameter = Some(ForParameter::MultiVariableDeclaration(
                    MultiVariableDeclaration::new(&child, content)?,
                ))
            }
            "control_structure_body" => {
                body = Some(ControlStructureBody::new(&child, content)?);
            }
            kind => {
                if EXPRESSIONS.contains(&kind) {
                    expression = Some(Expression::new(&child, content)?);
                }
            }
        }
    }

    Ok(Statement::For(
        expression.context(format!(
            "[Statement::For] no expression at {} - {}",
            node.start_position(),
            node.end_position()
        ))?,
        parameter.context(format!(
            "[Statement::For] no parameter at {} - {}",
            node.start_position(),
            node.end_position()
        ))?,
        body,
    ))
}
