use crate::kotlin::types::Type;
use anyhow::{bail, Context, Result};
use tree_sitter::Node;

use super::{
    expression::Expression,
    statement::{self, Statement},
    types::TYPES,
};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum FunctionModifier {
    Annotation(String),
    Member(String),
    Visibility(String),
    Function(String),
    Inheritance(String),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Parameter {
    pub name: String,
    pub type_identifier: Type,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Identifier {
    pub name: String,
    pub data_type: Option<Type>,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum FunctionBody {
    Block(Vec<Statement>),
    Expression(Expression),
}

impl FunctionBody {
    pub fn new(node: &Node, content: &[u8]) -> Result<FunctionBody> {
        let first = node.child(0).context(format!(
            "[FunctionBody] no child at {}",
            node.start_position()
        ))?;
        let second = node.child(1).context(format!(
            "[FunctionBody] no child at {}",
            node.start_position()
        ))?;

        Ok(match first.kind() {
            "=" => FunctionBody::Expression(Expression::new(&second, content)?),
            _ => FunctionBody::Block(statement::get_statements(&second, content)?),
        })
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Function {
    pub modifiers: Vec<FunctionModifier>,
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<String>,
    pub body: Option<FunctionBody>,
}

impl Function {
    pub fn new(node: &Node, content: &[u8]) -> Result<Function> {
        let mut modifiers: Vec<FunctionModifier> = Vec::new();
        let mut parameters: Vec<Parameter> = Vec::new();
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
                        parameters.push(Parameter {
                            name: child
                                .child(0)
                                .context("no parameter name found")?
                                .utf8_text(content)?
                                .to_string(),
                            type_identifier: Type::new(
                                &child
                                    .child(2)
                                    .filter(|c| c.kind() != "type_modifiers")
                                    .or_else(|| child.child(3))
                                    .context("no type identifier found")?,
                                content,
                            )?,
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

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct ParameterWithOptionalType {
    identifier: String,
    data_type: Option<Type>,
}

impl ParameterWithOptionalType {
    pub fn new(node: &Node, content: &[u8]) -> Result<ParameterWithOptionalType> {
        let mut identifier = None;
        let mut data_type = None;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "simple_identifier" => identifier = Some(child.utf8_text(content)?.to_string()),
                kind => {
                    if TYPES.contains(&kind) {
                        data_type = Some(Type::new(&child, content)?);
                    } else {
                        bail!(
                            "[ParameterWithOptionalType] unhandled child {} '{}' at {}",
                            child.kind(),
                            child.utf8_text(content)?,
                            child.start_position(),
                        )
                    }
                }
            }
        }

        Ok(ParameterWithOptionalType {
            identifier: identifier.context(format!(
                "[ParameterWithOptionalType] no identifier found at {}",
                node.start_position()
            ))?,
            data_type,
        })
    }
}
