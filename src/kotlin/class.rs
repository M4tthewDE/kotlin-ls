use crate::kotlin::modifier::Modifier;
use anyhow::{bail, Context, Result};
use tree_sitter::{Node, Tree};

use super::{delegation::Delegation, function::Function, object::Object, property::Property, Type};

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum ClassBody {
    Class {
        properties: Vec<Property>,
        functions: Vec<Function>,
        objects: Vec<Object>,
        classes: Vec<Class>,
        companion_objects: Vec<CompanionObject>,
    },
}

impl ClassBody {
    fn new_class_body(node: &Node, content: &[u8]) -> Result<ClassBody> {
        let mut properties: Vec<Property> = Vec::new();
        let mut functions: Vec<Function> = Vec::new();
        let mut objects: Vec<Object> = Vec::new();
        let mut classes: Vec<Class> = Vec::new();
        let mut companion_objects = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "{" | "}" | "line_comment" | "getter" | "setter" => {}
                "property_declaration" => {
                    properties.push(Property::new(&child, content)?);
                }
                "function_declaration" => {
                    functions.push(Function::new(&child, content)?);
                }
                "object_declaration" => {
                    objects.push(Object::new(&child, content)?);
                }
                "class_declaration" => {
                    classes.push(Class::new(&child, content)?);
                }
                "companion_object" => {
                    companion_objects.push(CompanionObject::new(&child, content)?);
                }
                _ => {
                    bail!(
                        "[ClassBody::Class] unhandled child {} '{}' at {}",
                        child.kind(),
                        child.utf8_text(content)?,
                        child.start_position(),
                    )
                }
            }
        }

        Ok(ClassBody::Class {
            properties,
            functions,
            objects,
            classes,
            companion_objects,
        })
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct CompanionObject {
    body: ClassBody,
}

impl CompanionObject {
    fn new(node: &Node, content: &[u8]) -> Result<CompanionObject> {
        let mut body = None;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "companion" | "object" => {}
                "class_body" => body = Some(ClassBody::new_class_body(&child, content)?),
                _ => {
                    bail!(
                        "[ClassBody::CompanionObject] unhandled child {} '{}' at {}",
                        child.kind(),
                        child.utf8_text(content)?,
                        child.start_position(),
                    )
                }
            }
        }

        Ok(CompanionObject {
            body: body.context("no class body found")?,
        })
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum ClassParameterMutability {
    Val,
    Var,
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct ClassParameter {
    mutability: ClassParameterMutability,
    name: String,
    data_type: Type,
    modifiers: Vec<Modifier>,
}

impl ClassParameter {
    fn new(node: &Node, content: &[u8]) -> Result<ClassParameter> {
        let mut mutability = None;
        let mut name = None;
        let mut data_type = None;
        let mut modifiers = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor.clone()) {
            match child.kind() {
                "val" => mutability = Some(ClassParameterMutability::Val),
                "var" => mutability = Some(ClassParameterMutability::Var),
                "modifiers" => {
                    for child in child.children(&mut cursor) {
                        modifiers.push(Modifier::new(&child, content)?);
                    }
                }
                "simple_identifier" => name = Some(child.utf8_text(content)?.to_string()),
                "user_type" => {
                    data_type = Some(Type::NonNullable(child.utf8_text(content)?.to_string()))
                }
                "nullable_type" => {
                    data_type = Some(Type::Nullable(child.utf8_text(content)?.to_string()))
                }
                ":" => {}
                _ => {
                    bail!(
                        "[ClassParameter] unhandled child {} '{}' at {}",
                        child.kind(),
                        child.utf8_text(content)?,
                        child.start_position(),
                    )
                }
            }
        }

        Ok(ClassParameter {
            mutability: mutability.context("no mutability found")?,
            name: name.context("no name found")?,
            data_type: data_type.context("no data_type found")?,
            modifiers,
        })
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Constructor {
    modifiers: Vec<Modifier>,
    parameters: Vec<ClassParameter>,
}

impl Constructor {
    fn new(node: &Node, content: &[u8]) -> Result<Constructor> {
        let mut modifiers = Vec::new();
        let mut parameters = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor.clone()) {
            match child.kind() {
                "(" | "," | ")" | "constructor" => {}
                "modifiers" => {
                    for child in child.children(&mut cursor) {
                        modifiers.push(Modifier::new(&child, content)?);
                    }
                }
                "class_parameter" => parameters.push(ClassParameter::new(&child, content)?),
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

        Ok(Constructor {
            parameters,
            modifiers,
        })
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum ClassType {
    Class,
    Interface,
    Enum,
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Class {
    pub class_type: ClassType,
    pub name: Type,
    pub modifiers: Vec<Modifier>,
    pub constructor: Option<Constructor>,
    pub delegations: Vec<Delegation>,
    pub body: Option<ClassBody>,
}

impl Class {
    fn new(node: &Node, content: &[u8]) -> Result<Class> {
        let mut modifiers = Vec::new();
        let mut class_type = None;
        let mut name = None;
        let mut constructor = None;
        let mut body = None;
        let mut delegations = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor.clone()) {
            match child.kind() {
                ":" => {}
                "modifiers" => {
                    for child in child.children(&mut cursor) {
                        modifiers.push(Modifier::new(&child, content)?);
                    }
                }
                "class" => class_type = Some(ClassType::Class),
                "interface" => class_type = Some(ClassType::Interface),
                "enum" => class_type = Some(ClassType::Enum),
                "type_identifier" => {
                    name = Some(Type::NonNullable(child.utf8_text(content)?.to_string()))
                }
                "primary_constructor" => constructor = Some(Constructor::new(&child, content)?),
                "delegation_specifier" => delegations.push(Delegation::new(&child, content)?),
                "class_body" => body = Some(ClassBody::new_class_body(&child, content)?),
                _ => {
                    bail!(
                        "KotlinClass: unhandled child {} '{}' at {}",
                        child.kind(),
                        child.utf8_text(content)?,
                        child.start_position(),
                    )
                }
            }
        }

        Ok(Class {
            class_type: class_type.context("no class type found")?,
            name: name.context("no class name found")?,
            modifiers,
            delegations,
            constructor,
            body,
        })
    }
}

pub fn get_classes(tree: &Tree, content: &[u8]) -> Result<Vec<Class>> {
    let mut classes = Vec::new();
    let mut cursor = tree.walk();
    loop {
        let node = cursor.node();
        if node.kind() == "class_declaration" {
            classes.push(Class::new(&node, content)?);
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