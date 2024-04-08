use anyhow::{bail, Result};
use tree_sitter::Node;

use super::{
    class::ClassBody,
    delegation::Delegation,
    statement::{self, Statement},
    variable_declaration::VariableDeclaration,
};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Literal {
    Boolean(String),
    String(String),
    Integer(String),
    Object(ClassBody, Vec<Delegation>),
    Character(String),
    Lambda(Option<Vec<Statement>>, Option<Vec<LambdaParameter>>),
    Null,
}

impl Literal {
    pub fn new(node: &Node, content: &[u8]) -> Result<Literal> {
        match node.kind() {
            "boolean_literal" => Ok(Literal::Boolean(node.utf8_text(content)?.to_string())),
            "string_literal" => Ok(Literal::String(node.utf8_text(content)?.to_string())),
            "integer_literal" => Ok(Literal::Integer(node.utf8_text(content)?.to_string())),
            "character_literal" => Ok(Literal::Character(node.utf8_text(content)?.to_string())),
            "object_literal" => {
                let mut delegations = Vec::new();
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    match child.kind() {
                        "object" | ":" => {}
                        "delegation_specifier" => {
                            delegations.push(Delegation::new(&child, content)?)
                        }
                        "class_body" => {
                            return Ok(Literal::Object(
                                ClassBody::new_class_body(&child, content)?,
                                delegations,
                            ))
                        }
                        _ => {
                            bail!(
                                "[Literal] unhandled node {} '{}' at {}",
                                child.kind(),
                                child.utf8_text(content)?,
                                child.start_position(),
                            )
                        }
                    }
                }

                bail!("[Literal] no class_body at {}", node.start_position());
            }
            "lambda_literal" => {
                let mut statements = None;
                let mut parameters = None;
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    match child.kind() {
                        "{" | "->" | "}" => {}
                        "statements" => {
                            statements = Some(statement::get_statements(&child, content)?)
                        }
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

                Ok(Literal::Lambda(statements, parameters))
            }
            "null" => Ok(Literal::Null),
            _ => {
                bail!(
                    "[Literal] unhandled node {} '{}' at {}",
                    node.kind(),
                    node.utf8_text(content)?,
                    node.start_position(),
                )
            }
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum LambdaParameter {
    VariableDeclaration(VariableDeclaration),
}

fn get_parameters(node: &Node, content: &[u8]) -> Result<Vec<LambdaParameter>> {
    let mut parameters = Vec::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "," => {}
            "variable_declaration" => parameters.push(LambdaParameter::VariableDeclaration(
                VariableDeclaration::new(&child, content)?,
            )),
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
