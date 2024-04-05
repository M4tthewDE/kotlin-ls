use anyhow::{bail, Context, Result};
use tree_sitter::Node;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum PropertyModifier {
    Annotation(String),
    Member(String),
    Visibility(String),
    Inheritance(String),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum PropertyMutability {
    Var,
    Val,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Property {
    pub modifiers: Vec<PropertyModifier>,
    pub name: String,
    pub data_type: Option<String>,
    pub extension_type: Option<String>,
    pub mutability: PropertyMutability,
}

impl Property {
    pub fn new(node: &Node, content: &[u8]) -> Result<Property> {
        let mut modifiers: Vec<PropertyModifier> = Vec::new();
        let mut mutability = None;
        let mut extension_type = None;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor.clone()) {
            match child.kind() {
                "modifiers" => {
                    for child in child.children(&mut cursor) {
                        match child.kind() {
                            "annotation" => modifiers.push(PropertyModifier::Annotation(
                                child.utf8_text(content)?.to_string(),
                            )),
                            "member_modifier" => modifiers.push(PropertyModifier::Member(
                                child.utf8_text(content)?.to_string(),
                            )),
                            "visibility_modifier" => modifiers.push(PropertyModifier::Visibility(
                                child.utf8_text(content)?.to_string(),
                            )),
                            "inheritance_modifier" => {
                                modifiers.push(PropertyModifier::Inheritance(
                                    child.utf8_text(content)?.to_string(),
                                ))
                            }
                            _ => bail!("unknown modifier {}", child.kind()),
                        }
                    }
                }
                "var" => mutability = Some(PropertyMutability::Var),
                "val" => mutability = Some(PropertyMutability::Val),
                "user_type" => extension_type = Some(child.utf8_text(content)?.to_string()),
                "variable_declaration" => {
                    let name = child
                        .child(0)
                        .context("no name found for variable declaration")?
                        .utf8_text(content)?
                        .to_string();
                    let data_type = if let Some(type_node) = child.child(2) {
                        Some(type_node.utf8_text(content)?.to_string())
                    } else {
                        None
                    };

                    return Ok(Property {
                        modifiers,
                        name,
                        data_type,
                        extension_type,
                        mutability: mutability.context("no mutability modifier found")?,
                    });
                }
                "." => {}
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

        bail!("no property found");
    }
}
