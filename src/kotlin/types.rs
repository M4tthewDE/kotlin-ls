use anyhow::{bail, Context, Result};
use tree_sitter::Node;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct FunctionTypeParameter {
    identifier: String,
    param_type: Type,
}

impl FunctionTypeParameter {
    pub fn new(node: &Node, content: &[u8]) -> Result<FunctionTypeParameter> {
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

        Ok(FunctionTypeParameter {
            identifier: identifier.context(format!(
                "[FunctionTypeParameter] no identifier found at {}",
                node.start_position()
            ))?,
            param_type: param_type.context(format!(
                "[FunctionTypeParameter] no param type found at {}",
                node.start_position()
            ))?,
        })
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Type {
    Nullable(String),
    NonNullable(String),
    Function {
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
    let mut parameters = None;
    let mut return_type = None;
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "->" => {}
            "function_type_parameters" => {
                let mut params = Vec::new();
                let mut cursor = node.walk();
                for child in child.children(&mut cursor) {
                    match child.kind() {
                        "(" | ")" => {}
                        "parameter" => params.push(FunctionTypeParameter::new(&child, content)?),
                        _ => {
                            bail!(
                                "[Type::Function::Params] unhandled child {} '{}' at {}",
                                child.kind(),
                                child.utf8_text(content)?,
                                child.start_position(),
                            )
                        }
                    }
                }

                parameters = Some(params);
            }
            "user_type" => return_type = Some(Type::new(&child, content)?),
            _ => {
                bail!(
                    "[Type::Function] unhandled child {} '{}' at {}",
                    child.kind(),
                    child.utf8_text(content)?,
                    child.start_position(),
                )
            }
        }
    }

    Ok(Type::Function {
        parameters: parameters.context(format!(
            "[Type::Function] no parameters found at {}",
            node.start_position()
        ))?,
        return_type: Box::new(return_type.context(format!(
            "[Type::Function] no return type at {}",
            node.start_position()
        ))?),
    })
}
