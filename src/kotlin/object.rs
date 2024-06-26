use anyhow::{bail, Context, Result};
use tree_sitter::Node;

use super::{class::ClassBody, delegation::Delegation, modifier::Modifier};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Object {
    pub modifiers: Vec<Modifier>,
    pub name: String,
    pub delegations: Vec<Delegation>,
    pub class_body: Option<ClassBody>,
}

impl Object {
    pub fn new(node: &Node, content: &[u8]) -> Result<Object> {
        let mut modifiers = Vec::new();
        let mut name = None;
        let mut delegations = Vec::new();
        let mut class_body = None;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor.clone()) {
            match child.kind() {
                "object" | ":" => {}
                "modifiers" => {
                    for child in child.children(&mut cursor) {
                        modifiers.push(Modifier::new(&child, content)?);
                    }
                }
                "type_identifier" => name = Some(child.utf8_text(content)?.to_string()),
                "delegation_specifier" => delegations.push(Delegation::new(&child, content)?),
                "class_body" => class_body = Some(ClassBody::new_class_body(&child, content)?),
                _ => {
                    bail!(
                        "[Object] unhandled child {} '{}' at {}",
                        child.kind(),
                        child.utf8_text(content)?,
                        child.start_position(),
                    )
                }
            }
        }

        Ok(Object {
            modifiers,
            name: name.context("no name found")?,
            delegations,
            class_body,
        })
    }
}
