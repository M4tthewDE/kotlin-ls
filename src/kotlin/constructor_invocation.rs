use anyhow::{bail, Context, Result};
use tree_sitter::Node;

use super::{
    argument::{self, ValueArgument},
    types::Type,
};

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct ConstructorInvocation {
    data_type: Type,
    arguments: Vec<ValueArgument>,
}

impl ConstructorInvocation {
    pub fn new(node: &Node, content: &[u8]) -> Result<ConstructorInvocation> {
        let mut data_type = None;
        let mut arguments = None;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "user_type" => data_type = Some(Type::new(&child, content)?),
                "value_arguments" => arguments = Some(argument::get_arguments(&child, content)?),
                _ => {
                    bail!(
                        "unhandled child {} '{}' at {}",
                        child.kind(),
                        child.utf8_text(content)?,
                        child.start_position(),
                    )
                }
            }
        }

        Ok(ConstructorInvocation {
            data_type: data_type.context("no data type found")?,
            arguments: arguments.context("no arguments found")?,
        })
    }
}
