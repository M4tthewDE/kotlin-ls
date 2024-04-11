use anyhow::{bail, Result};
use tree_sitter::Node;

use crate::kotlin::function::FunctionBody;

use super::{function::ParameterWithOptionalType, modifier::Modifier};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Getter {
    modifiers: Option<Vec<Modifier>>,
    function_body: Option<FunctionBody>,
}

impl Getter {
    pub fn new(node: &Node, content: &[u8]) -> Result<Getter> {
        let mut function_body = None;
        let mut modifiers = None;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "get" | "(" | ")" => {}
                "modifiers" => {
                    let mut m = Vec::new();
                    let mut cursor = node.walk();
                    for child in child.children(&mut cursor) {
                        m.push(Modifier::new(&child, content)?);
                    }
                    modifiers = Some(m)
                }
                "function_body" => function_body = Some(FunctionBody::new(&child, content)?),
                _ => {
                    bail!(
                        "[Getter] unhandled child {} '{}' at {}",
                        child.kind(),
                        child.utf8_text(content)?,
                        child.start_position(),
                    )
                }
            }
        }

        Ok(Getter {
            modifiers,
            function_body,
        })
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Setter {
    modifiers: Option<Vec<Modifier>>,
    parameter: Option<ParameterWithOptionalType>,
    function_body: Option<FunctionBody>,
}

impl Setter {
    pub fn new(node: &Node, content: &[u8]) -> Result<Setter> {
        let mut parameter = None;
        let mut modifiers = None;
        let mut function_body = None;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "set" | "(" | ")" => {}
                "modifiers" => {
                    let mut m = Vec::new();
                    let mut cursor = node.walk();
                    for child in child.children(&mut cursor) {
                        m.push(Modifier::new(&child, content)?);
                    }
                    modifiers = Some(m)
                }
                "function_body" => function_body = Some(FunctionBody::new(&child, content)?),
                "parameter_with_optional_type" => {
                    parameter = Some(ParameterWithOptionalType::new(&child, content)?)
                }
                _ => {
                    bail!(
                        "[Setter] unhandled child {} '{}' at {}",
                        child.kind(),
                        child.utf8_text(content)?,
                        child.start_position(),
                    )
                }
            }
        }

        Ok(Setter {
            modifiers,
            parameter,
            function_body,
        })
    }
}
