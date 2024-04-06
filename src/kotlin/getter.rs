use anyhow::{bail, Context, Result};
use tree_sitter::Node;

use crate::kotlin::function::FunctionBody;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Getter {
    function_body: FunctionBody,
}

impl Getter {
    pub fn new(node: &Node, content: &[u8]) -> Result<Getter> {
        let mut function_body = None;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "get" | "(" | ")" => {}
                "function_body" => function_body = Some(FunctionBody::new(&node, content)?),
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
            function_body: function_body.context("no function body found")?,
        })
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Setter {
    function_body: FunctionBody,
}

impl Setter {
    pub fn new(node: &Node, content: &[u8]) -> Result<Setter> {
        let function_body = None;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                //"get" | "(" | ")" => {}
                //"function_body" => function_body = Some(FunctionBody::new(&node, content)?),
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

        Ok(Setter {
            function_body: function_body.context("no function body found")?,
        })
    }
}
