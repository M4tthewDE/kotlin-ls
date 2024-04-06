use anyhow::{bail, Context, Result};
use tree_sitter::{Node, Tree};

use self::{function::Function, property::Property};

use super::{
    argument::{self, ValueArgument},
    Type,
};

mod function;
mod property;

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum Modifier {
    Class(String),
    Visibility(String),
    Annotation(String),
    Inheritance(String),
}

impl Modifier {
    fn new(node: &Node, content: &[u8]) -> Result<Modifier> {
        match node.kind() {
            "visibility_modifier" => Ok(Modifier::Visibility(node.utf8_text(content)?.to_string())),
            "class_modifier" => Ok(Modifier::Class(node.utf8_text(content)?.to_string())),
            "annotation" => Ok(Modifier::Annotation(node.utf8_text(content)?.to_string())),
            "inheritance_modifier" => {
                Ok(Modifier::Inheritance(node.utf8_text(content)?.to_string()))
            }
            _ => bail!("unknown modifier {}", node.kind()),
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct ClassBody {
    pub properties: Vec<Property>,
    pub functions: Vec<Function>,
}

impl ClassBody {
    fn new(node: &Node, content: &[u8]) -> Result<ClassBody> {
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
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum ClassParameterMutability {
    Val,
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
                        "unhandled child {} '{}' at {}",
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
pub struct ConstructorInvocation {
    data_type: Type,
    arguments: Vec<ValueArgument>,
}

impl ConstructorInvocation {
    fn new(node: &Node, content: &[u8]) -> Result<ConstructorInvocation> {
        let mut data_type = None;
        let mut arguments = None;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "user_type" => {
                    data_type = Some(Type::NonNullable(child.utf8_text(content)?.to_string()))
                }
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

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum Delegation {
    Type(Type),
    ConstructorInvocation(ConstructorInvocation),
}

impl Delegation {
    fn new(node: &Node, content: &[u8]) -> Result<Delegation> {
        let child = node.child(0).context("no delegation specifier child")?;
        match child.kind() {
            "user_type" => Ok(Delegation::Type(Type::NonNullable(
                child.utf8_text(content)?.to_string(),
            ))),
            "constructor_invocation" => Ok(Delegation::ConstructorInvocation(
                ConstructorInvocation::new(&child, content)?,
            )),
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
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum ClassType {
    Class,
    Interface,
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct KotlinClass {
    pub class_type: ClassType,
    pub name: Type,
    pub modifiers: Vec<Modifier>,
    pub constructor: Option<Constructor>,
    pub delegations: Vec<Delegation>,
    pub body: Option<ClassBody>,
}

impl KotlinClass {
    fn new(node: &Node, content: &[u8]) -> Result<KotlinClass> {
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
                "type_identifier" => {
                    name = Some(Type::NonNullable(child.utf8_text(content)?.to_string()))
                }
                "primary_constructor" => constructor = Some(Constructor::new(&child, content)?),
                "delegation_specifier" => delegations.push(Delegation::new(&child, content)?),
                "class_body" => body = Some(ClassBody::new(&child, content)?),
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

        Ok(KotlinClass {
            class_type: class_type.context("no class type found")?,
            name: name.context("no class name found")?,
            modifiers,
            delegations,
            constructor,
            body,
        })
    }
}

pub fn get_classes(tree: &Tree, content: &[u8]) -> Result<Vec<KotlinClass>> {
    let mut classes = Vec::new();
    let mut cursor = tree.walk();
    loop {
        let node = cursor.node();
        if node.kind() == "class_declaration" {
            classes.push(KotlinClass::new(&node, content)?);
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