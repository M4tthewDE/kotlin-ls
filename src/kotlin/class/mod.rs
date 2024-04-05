use anyhow::{bail, Context, Result};
use tree_sitter::{Node, Tree};

use self::function::Function;

use super::DataType;

mod function;

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum ClassModifier {
    Class(String),
    Visibility(String),
    Annotation(String),
    Inheritance(String),
}

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
    pub data_type: Option<DataType>,
    pub extension_type: Option<DataType>,
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
                "user_type" => {
                    extension_type = Some(DataType(child.utf8_text(content)?.to_string()))
                }
                "variable_declaration" => {
                    let name = child
                        .child(0)
                        .context("no name found for variable declaration")?
                        .utf8_text(content)?
                        .to_string();
                    let data_type = if let Some(type_node) = child.child(2) {
                        Some(DataType(type_node.utf8_text(content)?.to_string()))
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

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct ClassBody {
    pub properties: Vec<Property>,
    pub functions: Vec<Function>,
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct KotlinClass {
    pub name: String,
    pub modifiers: Vec<ClassModifier>,
    pub supertypes: Vec<String>,
    pub body: ClassBody,
}

pub fn get_classes(tree: &Tree, content: &[u8]) -> Result<Vec<KotlinClass>> {
    let mut classes = Vec::new();
    let mut cursor = tree.walk();
    loop {
        let node = cursor.node();
        if node.kind() == "class_declaration" {
            let name = get_class_name(&node, content)?;
            let modifiers = get_class_modifiers(&node, content)?;
            let supertypes = get_supertypes(&node, content)?;
            let body = get_class_body(&node, content)?;
            classes.push(KotlinClass {
                name,
                modifiers,
                supertypes,
                body,
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
                return Ok(classes);
            }
        }
    }
}

fn get_class_name(node: &Node, content: &[u8]) -> Result<String> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "type_identifier" {
            return Ok(child
                .utf8_text(content)
                .context("malformed class")?
                .to_string());
        }
    }

    bail!("no class name found");
}

fn get_class_modifiers(node: &Node, content: &[u8]) -> Result<Vec<ClassModifier>> {
    let mut modifiers: Vec<ClassModifier> = Vec::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor.clone()) {
        if child.kind() == "modifiers" {
            for child in child.children(&mut cursor) {
                match child.kind() {
                    "visibility_modifier" => modifiers.push(ClassModifier::Visibility(
                        child.utf8_text(content)?.to_string(),
                    )),
                    "class_modifier" => {
                        modifiers.push(ClassModifier::Class(child.utf8_text(content)?.to_string()))
                    }
                    "annotation" => modifiers.push(ClassModifier::Annotation(
                        child.utf8_text(content)?.to_string(),
                    )),
                    "inheritance_modifier" => modifiers.push(ClassModifier::Inheritance(
                        child.utf8_text(content)?.to_string(),
                    )),
                    _ => bail!("unknown modifier {}", child.kind()),
                }
            }
        }
    }

    Ok(modifiers)
}

fn get_supertypes(node: &Node, content: &[u8]) -> Result<Vec<String>> {
    let mut supertypes: Vec<String> = Vec::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "delegation_specifier" {
            supertypes.push(child.utf8_text(content)?.to_string());
        }
    }

    Ok(supertypes)
}

fn get_class_body(node: &Node, content: &[u8]) -> Result<ClassBody> {
    let mut properties: Vec<Property> = Vec::new();
    let mut functions: Vec<Function> = Vec::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor.clone()) {
        if child.kind() == "class_body" {
            for child in child.children(&mut cursor) {
                if child.kind() == "property_declaration" {
                    properties.push(Property::new(&child, content)?);
                }

                if child.kind() == "function_declaration" {
                    functions.push(Function::new(&child, content)?);
                }
            }
        }
    }

    Ok(ClassBody {
        properties,
        functions,
    })
}
