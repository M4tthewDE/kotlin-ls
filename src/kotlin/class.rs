use crate::kotlin::modifier::Modifier;
use anyhow::{bail, Context, Result};
use tree_sitter::{Node, Tree};

use super::{
    argument::{self, Argument},
    delegation::Delegation,
    expression::Expression,
    function::{Function, Parameter},
    object::Object,
    property::Property,
    statement::{self, Statement},
    types::{Type, TypeParameter},
};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct EnumEntry {
    modifiers: Vec<Modifier>,
    identifier: String,
    value_arguments: Option<Vec<Argument>>,
    class_body: Option<ClassBody>,
}

impl EnumEntry {
    fn new(node: &Node, content: &[u8]) -> Result<EnumEntry> {
        let mut identifier = None;
        let mut value_arguments = None;
        let mut modifiers = Vec::new();
        let mut class_body = None;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor.clone()) {
            match child.kind() {
                "modifiers" => {
                    for child in child.children(&mut cursor) {
                        modifiers.push(Modifier::new(&child, content)?);
                    }
                }
                "simple_identifier" => identifier = Some(child.utf8_text(content)?.to_string()),
                "value_arguments" => {
                    value_arguments = Some(argument::get_value_arguments(&child, content)?)
                }
                "class_body" => class_body = Some(ClassBody::new_class_body(&child, content)?),
                _ => {}
            }
        }

        Ok(EnumEntry {
            modifiers,
            identifier: identifier.context(format!(
                "[EnumEntry] no identifier at {}",
                node.start_position()
            ))?,
            value_arguments,
            class_body,
        })
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct AnonymousInitializer {
    statements: Vec<Statement>,
}

impl AnonymousInitializer {
    fn new(node: &Node, content: &[u8]) -> Result<AnonymousInitializer> {
        let mut statements = None;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "init" | "{" | "}" => {}
                "statements" => statements = Some(statement::get_statements(&child, content)?),
                _ => {
                    bail!(
                        "[AnonymousInitializer] unhandled child {} '{}' at {}",
                        child.kind(),
                        child.utf8_text(content)?,
                        child.start_position(),
                    )
                }
            }
        }

        Ok(AnonymousInitializer {
            statements: statements.context(format!(
                "[AnonymousInitializer] no statements at {}",
                node.start_position()
            ))?,
        })
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum ClassBody {
    Class {
        properties: Vec<Property>,
        functions: Vec<Function>,
        objects: Vec<Object>,
        classes: Vec<Class>,
        companion_objects: Vec<CompanionObject>,
        anonymous_initializers: Vec<AnonymousInitializer>,
        secondary_constructors: Vec<SecondaryConstructor>,
    },
    Enum {
        entries: Vec<EnumEntry>,
        properties: Vec<Property>,
        functions: Vec<Function>,
        objects: Vec<Object>,
        classes: Vec<Class>,
        companion_objects: Vec<CompanionObject>,
        anonymous_initializers: Vec<AnonymousInitializer>,
        secondary_constructors: Vec<SecondaryConstructor>,
    },
}

impl ClassBody {
    pub fn new_class_body(node: &Node, content: &[u8]) -> Result<ClassBody> {
        let mut properties: Vec<Property> = Vec::new();
        let mut functions: Vec<Function> = Vec::new();
        let mut objects: Vec<Object> = Vec::new();
        let mut classes: Vec<Class> = Vec::new();
        let mut companion_objects = Vec::new();
        let mut anonymous_initializers = Vec::new();
        let mut secondary_constructors = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "{" | "}" | "line_comment" | "multiline_comment" | "getter" | "setter" => {}
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
                "anonymous_initializer" => {
                    anonymous_initializers.push(AnonymousInitializer::new(&child, content)?);
                }
                "secondary_constructor" => {
                    secondary_constructors.push(SecondaryConstructor::new(&child, content)?);
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
            anonymous_initializers,
            secondary_constructors,
        })
    }

    fn new_enum_class_body(node: &Node, content: &[u8]) -> Result<ClassBody> {
        let mut entries = Vec::new();
        let mut properties: Vec<Property> = Vec::new();
        let mut functions: Vec<Function> = Vec::new();
        let mut objects: Vec<Object> = Vec::new();
        let mut classes: Vec<Class> = Vec::new();
        let mut companion_objects = Vec::new();
        let mut anonymous_initializers = Vec::new();
        let mut secondary_constructors = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "{" | "," | "}" | ";" | "getter" | "setter" | "line_comment" => {}
                "enum_entry" => entries.push(EnumEntry::new(&child, content)?),
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
                "anonymous_initializer" => {
                    anonymous_initializers.push(AnonymousInitializer::new(&child, content)?);
                }
                "secondary_constructor" => {
                    secondary_constructors.push(SecondaryConstructor::new(&child, content)?);
                }
                _ => {
                    bail!(
                        "[ClassBody::Enum] unhandled child {} '{}' at {}",
                        child.kind(),
                        child.utf8_text(content)?,
                        child.start_position(),
                    )
                }
            }
        }

        Ok(ClassBody::Enum {
            entries,
            properties,
            functions,
            objects,
            classes,
            companion_objects,
            anonymous_initializers,
            secondary_constructors,
        })
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
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

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum ClassParameterMutability {
    Val,
    Var,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct ClassParameter {
    mutability: Option<ClassParameterMutability>,
    name: String,
    data_type: Type,
    modifiers: Vec<Modifier>,
    expression: Option<Expression>,
}

impl ClassParameter {
    fn new(node: &Node, content: &[u8]) -> Result<ClassParameter> {
        let mut mutability = None;
        let mut name = None;
        let mut data_type = None;
        let mut modifiers = Vec::new();
        let mut expression = None;
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
                "user_type" | "nullable_type" | "function_type" => {
                    data_type = Some(Type::new(&child, content)?)
                }
                ":" => {}
                "=" => {
                    expression = Some(Expression::new(
                        &child.next_sibling().context(format!(
                            "[ClassParameter] no sibling at {}",
                            child.start_position()
                        ))?,
                        content,
                    )?);
                    break;
                }
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
            mutability,
            name: name.context(format!(
                "[ClassParameter] no name found at {}",
                node.start_position()
            ))?,
            data_type: data_type.context(format!(
                "[ClassParameter] no data type found at {}",
                node.start_position()
            ))?,
            modifiers,
            expression,
        })
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
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
                "(" | "," | ")" | "constructor" | "line_comment" => {}
                "modifiers" => {
                    for child in child.children(&mut cursor) {
                        modifiers.push(Modifier::new(&child, content)?);
                    }
                }
                "class_parameter" => parameters.push(ClassParameter::new(&child, content)?),
                _ => {
                    bail!(
                        "[Constructor] unhandled child {} '{}' at {}",
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

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum ClassType {
    Class,
    Interface,
    Enum,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Class {
    pub class_type: ClassType,
    pub name: String,
    pub modifiers: Vec<Modifier>,
    pub type_parameters: Vec<TypeParameter>,
    pub constructor: Option<Constructor>,
    pub delegations: Vec<Delegation>,
    pub body: Option<ClassBody>,
}

impl Class {
    fn new(node: &Node, content: &[u8]) -> Result<Class> {
        let mut modifiers = Vec::new();
        let mut class_type = None;
        let mut name = None;
        let mut type_parameters = Vec::new();
        let mut constructor = None;
        let mut body = None;
        let mut delegations = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor.clone()) {
            match child.kind() {
                ":" | "," => {}
                "modifiers" => {
                    for child in child.children(&mut cursor) {
                        modifiers.push(Modifier::new(&child, content)?);
                    }
                }
                "class" => class_type = Some(ClassType::Class),
                "interface" => class_type = Some(ClassType::Interface),
                "enum" => class_type = Some(ClassType::Enum),
                "type_identifier" => name = Some(child.utf8_text(content)?.to_string()),
                "primary_constructor" => constructor = Some(Constructor::new(&child, content)?),
                "delegation_specifier" => delegations.push(Delegation::new(&child, content)?),
                "class_body" => body = Some(ClassBody::new_class_body(&child, content)?),
                "enum_class_body" => body = Some(ClassBody::new_enum_class_body(&child, content)?),
                "type_parameters" => {
                    for child in child.children(&mut cursor) {
                        if child.kind() == "type_parameter" {
                            type_parameters.push(TypeParameter::new(&child, content)?)
                        }
                    }
                }
                _ => {
                    bail!(
                        "[Class]: unhandled child {} '{}' at {}",
                        child.kind(),
                        child.utf8_text(content)?,
                        child.start_position(),
                    )
                }
            }
        }

        Ok(Class {
            class_type: class_type.context("[Class] no class type found")?,
            name: name.context("[Class] no class name found")?,
            modifiers,
            type_parameters,
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

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct SecondaryConstructor {
    pub parameters: Vec<Parameter>,
    pub block: Vec<Statement>,
}

impl SecondaryConstructor {
    fn new(node: &Node, content: &[u8]) -> Result<SecondaryConstructor> {
        let mut parameters = Vec::new();
        let mut block = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "statements" => block = statement::get_statements(&child, content)?,
                "function_value_parameters" => {
                    let mut cursor = child.walk();
                    for child in child.children(&mut cursor) {
                        if child.kind() == "parameter" {
                            parameters.push(Parameter {
                                name: child
                                    .child(0)
                                    .context("no parameter name found")?
                                    .utf8_text(content)?
                                    .to_string(),
                                type_identifier: Type::new(
                                    &child
                                        .child(2)
                                        .filter(|c| c.kind() != "type_modifiers")
                                        .or_else(|| child.child(3))
                                        .context("no type identifier found")?,
                                    content,
                                )?,
                            })
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(SecondaryConstructor { parameters, block })
    }
}
