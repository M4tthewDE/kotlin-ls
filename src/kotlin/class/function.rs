use anyhow::{bail, Context, Result};
use tree_sitter::Node;

use crate::kotlin::{Position, Type};

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum FunctionModifier {
    Annotation(String),
    Member(String),
    Visibility(String),
    Function(String),
    Inheritance(String),
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct FunctionParameter {
    pub name: String,
    pub type_identifier: Type,
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Identifier {
    pub name: String,
    pub range: (Position, Position),
    pub data_type: Option<Type>,
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct FunctionBody {
    pub identifiers: Vec<Identifier>,
}

impl FunctionBody {
    pub fn new(node: &Node, content: &[u8]) -> Result<FunctionBody> {
        let mut identifiers = Vec::new();
        let mut cursor = node.walk();
        loop {
            let node = cursor.node();

            if node.kind() == "simple_identifier" {
                let name = node.utf8_text(content)?.to_string();
                identifiers.push(Identifier {
                    name,
                    range: (
                        Position::new(node.start_position().row, node.start_position().column),
                        Position::new(node.end_position().row, node.end_position().column),
                    ),
                    data_type: None,
                });
            }

            if cursor.goto_first_child() {
                continue;
            }

            loop {
                if cursor.goto_next_sibling() {
                    break;
                }

                if !cursor.goto_parent() {
                    return Ok(FunctionBody { identifiers });
                }
            }
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Function {
    pub modifiers: Vec<FunctionModifier>,
    pub name: String,
    pub parameters: Vec<FunctionParameter>,
    pub return_type: Option<String>,
    pub body: Option<FunctionBody>,
}

impl Function {
    pub fn new(node: &Node, content: &[u8]) -> Result<Function> {
        let mut modifiers: Vec<FunctionModifier> = Vec::new();
        let mut parameters: Vec<FunctionParameter> = Vec::new();
        let mut name = None;
        let mut return_type = None;
        let mut body = None;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor.clone()) {
            if child.kind() == "modifiers" {
                for child in child.children(&mut cursor) {
                    match child.kind() {
                        "annotation" => modifiers.push(FunctionModifier::Annotation(
                            child.utf8_text(content)?.to_string(),
                        )),
                        "member_modifier" => modifiers.push(FunctionModifier::Member(
                            child.utf8_text(content)?.to_string(),
                        )),
                        "visibility_modifier" => modifiers.push(FunctionModifier::Visibility(
                            child.utf8_text(content)?.to_string(),
                        )),
                        "function_modifier" => modifiers.push(FunctionModifier::Function(
                            child.utf8_text(content)?.to_string(),
                        )),
                        "inheritance_modifier" => modifiers.push(FunctionModifier::Inheritance(
                            child.utf8_text(content)?.to_string(),
                        )),
                        _ => bail!("unknown modifier {}", child.kind()),
                    }
                }
            }

            if child.kind() == "simple_identifier" {
                name = Some(child.utf8_text(content)?.to_string());
            }

            if child.kind() == "function_value_parameters" {
                for child in child.children(&mut cursor) {
                    if child.kind() == "parameter" {
                        let name = child
                            .child(0)
                            .context("no parameter name found")?
                            .utf8_text(content)?
                            .to_string();

                        let type_identifier = child
                            .child(2)
                            .context("no type identifier found")?
                            .utf8_text(content)?
                            .to_string();

                        parameters.push(FunctionParameter {
                            name,
                            type_identifier: Type::Nullable(type_identifier),
                        })
                    }
                }
            }

            if child.kind() == "user_type" || child.kind() == "nullable_type" {
                return_type = Some(child.utf8_text(content)?.to_string());
            }

            if child.kind() == "function_body" {
                body = Some(FunctionBody::new(&child, content)?);
            }
        }

        Ok(Function {
            modifiers,
            name: name.context("no name found for function")?,
            parameters,
            return_type,
            body,
        })
    }
}
