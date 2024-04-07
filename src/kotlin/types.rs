use anyhow::{bail, Context, Result};
use tree_sitter::Node;

use super::{
    argument::{self, Argument},
    function::Parameter,
};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum FunctionTypeParameter {
    Parameter(Parameter),
    Type(Type),
}

impl FunctionTypeParameter {
    pub fn new_parameter(node: &Node, content: &[u8]) -> Result<FunctionTypeParameter> {
        let mut identifier = None;
        let mut param_type = None;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "(" | ")" | ":" => {}
                "simple_identifier" => identifier = Some(child.utf8_text(content)?.to_string()),
                "user_type" => param_type = Some(Type::new(&child, content)?),
                _ => {
                    bail!(
                        "[FunctionTypeParameter] unhandled child {} '{}' at {}",
                        child.kind(),
                        child.utf8_text(content)?,
                        child.start_position(),
                    )
                }
            }
        }

        Ok(FunctionTypeParameter::Parameter(Parameter {
            name: identifier.context(format!(
                "[FunctionTypeParameter] no identifier found at {}",
                node.start_position()
            ))?,
            type_identifier: param_type.context(format!(
                "[FunctionTypeParameter] no param type found at {}",
                node.start_position()
            ))?,
        }))
    }

    pub fn new_type(node: &Node, content: &[u8]) -> Result<FunctionTypeParameter> {
        Ok(FunctionTypeParameter::Type(Type::new(node, content)?))
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Type {
    Nullable(String),
    NonNullable(String),
    Function {
        type_identifier: Option<String>,
        type_argument: Option<Box<Argument>>,
        parameters: Vec<FunctionTypeParameter>,
        return_type: Box<Type>,
    },
}

impl Type {
    pub fn new(node: &Node, content: &[u8]) -> Result<Type> {
        match node.kind() {
            "function_type" => get_function_type(node, content),
            "user_type" => Ok(Type::NonNullable(node.utf8_text(content)?.to_string())),
            "nullable_type" => Ok(Type::Nullable(node.utf8_text(content)?.to_string())),
            _ => {
                bail!(
                    "[Type] unhandled type {} '{}' at {}",
                    node.kind(),
                    node.utf8_text(content)?,
                    node.start_position(),
                )
            }
        }
    }
}

fn get_function_type(node: &Node, content: &[u8]) -> Result<Type> {
    let first_child = node.child(0).context(format!(
        "[Type::Function] no function parameters found at {}",
        node.start_position(),
    ))?;
    let (type_identifier, type_argument, parameters, return_type) = match first_child.kind() {
        "function_type_parameters" => (
            None,
            None,
            get_function_type_params(&first_child, content)?,
            Box::new(Type::new(
                &node.child(2).context(format!(
                    "[Type::Function] no return type found at {}",
                    node.start_position(),
                ))?,
                content,
            )?),
        ),
        "type_identifier" => (
            Some(first_child.utf8_text(content)?.to_string()),
            Some(Box::new(argument::get_type_argument(
                &node.child(1).context(format!(
                    "[Type::Function] no function parameters found at {}",
                    node.start_position(),
                ))?,
                content,
            )?)),
            get_function_type_params(
                &node.child(2).context(format!(
                    "[Type::Function] no function parameters found at {}",
                    node.start_position(),
                ))?,
                content,
            )?,
            Box::new(Type::new(
                &node.child(5).context(format!(
                    "[Type::Function] no return type found at {}",
                    node.start_position(),
                ))?,
                content,
            )?),
        ),
        unknown_child => {
            bail!(
                "[Type::Function] unhandled child {unknown_child} at {}",
                node.start_position()
            )
        }
    };
    Ok(Type::Function {
        type_identifier,
        type_argument,
        parameters,
        return_type,
    })
}

fn get_function_type_params(node: &Node, content: &[u8]) -> Result<Vec<FunctionTypeParameter>> {
    let mut params = Vec::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "(" | ")" => {}
            "parameter" => params.push(FunctionTypeParameter::new_parameter(&child, content)?),
            "user_type" => params.push(FunctionTypeParameter::new_type(&child, content)?),
            _ => {
                bail!(
                    "[Type::Function::TypeParams] unhandled child {} '{}' at {}",
                    child.kind(),
                    child.utf8_text(content)?,
                    child.start_position(),
                )
            }
        }
    }

    Ok(params)
}
