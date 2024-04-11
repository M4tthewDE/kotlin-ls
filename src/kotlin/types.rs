use anyhow::{bail, Context, Result};
use tree_sitter::Node;

use super::{
    argument::{self, Argument},
    function::Parameter,
};

pub const TYPES: [&str; 6] = [
    "parenthesized_type",
    "nullable_type",
    "user_type",
    "dynamic",
    "function_type",
    "non_nullable_type",
];

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
                "user_type" | "nullable_type" => param_type = Some(Type::new(&child, content)?),
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
pub enum TypeModifier {
    Annotation(String),
    Suspend,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Type {
    Nullable(Vec<TypeModifier>, String),
    NonNullable(Vec<TypeModifier>, String),
    Function {
        modifiers: Vec<TypeModifier>,
        type_identifier: Option<String>,
        type_argument: Option<Box<Argument>>,
        parameters: Vec<FunctionTypeParameter>,
        return_type: Box<Type>,
    },
}

impl Type {
    pub fn new(node: &Node, content: &[u8]) -> Result<Type> {
        let modifiers = if let Some(prev) = node.prev_sibling() {
            let mut mods = Vec::new();
            let mut cursor = prev.walk();
            for child in prev.children(&mut cursor) {
                match child.kind() {
                    "annotation" => mods.push(TypeModifier::Annotation(
                        child.utf8_text(content)?.to_string(),
                    )),
                    "suspend" => mods.push(TypeModifier::Suspend),
                    _ => {
                        bail!(
                            "[Type::Modifier] unhandled modifier {} '{}' at {}",
                            child.kind(),
                            child.utf8_text(content)?,
                            child.start_position(),
                        )
                    }
                }
            }

            mods
        } else {
            Vec::new()
        };

        match node.kind() {
            "function_type" => get_function_type(modifiers, node, content),
            "user_type" => Ok(Type::NonNullable(
                modifiers,
                node.utf8_text(content)?.to_string(),
            )),
            "nullable_type" => Ok(Type::Nullable(
                modifiers,
                node.utf8_text(content)?.to_string(),
            )),
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

fn get_function_type(modifiers: Vec<TypeModifier>, node: &Node, content: &[u8]) -> Result<Type> {
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
                &node
                    .child(4)
                    .filter(|c| c.kind() != "->")
                    .or_else(|| node.child(5))
                    .context(format!(
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
        modifiers,
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
            "(" | ")" | "," => {}
            "parameter" => params.push(FunctionTypeParameter::new_parameter(&child, content)?),
            "user_type" | "nullable_type" => {
                params.push(FunctionTypeParameter::new_type(&child, content)?)
            }
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

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct TypeParameter {
    identifier: String,
    data_type: Option<Type>,
}

impl TypeParameter {
    pub fn new(node: &Node, content: &[u8]) -> Result<TypeParameter> {
        let mut identifier = None;
        let mut data_type = None;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "type_identifier" => identifier = Some(child.utf8_text(content)?.to_string()),
                kind => {
                    if TYPES.contains(&kind) {
                        data_type = Some(Type::new(&child, content)?);
                    } else {
                        bail!(
                            "[TypeParameter] unhandled child {} '{}' at {}",
                            child.kind(),
                            child.utf8_text(content)?,
                            child.start_position(),
                        )
                    }
                }
            }
        }

        Ok(TypeParameter {
            identifier: identifier.context(format!(
                "[TypeParameter] no identifier found at {}",
                node.start_position()
            ))?,
            data_type,
        })
    }
}
