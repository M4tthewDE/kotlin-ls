use anyhow::{bail, Context, Result};
use tower_lsp::lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind};
use tree_sitter::{Node, Tree};

use super::Position;

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum ClassModifier {
    Class(String),
    Visibility(String),
    Annotation(String),
    Inheritance(String),
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum PropertyModifier {
    Annotation(String),
    Member(String),
    Visibility(String),
    Inheritance(String),
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Property {
    pub modifiers: Vec<PropertyModifier>,
    pub name: String,
    pub type_identifier: Option<String>,
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum FunctionModifier {
    Annotation(String),
    Member(String),
    Visibility(String),
    Function(String),
    Inheritance(String),
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct FunctionParameter {
    pub name: String,
    pub type_identifier: String,
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Identifier {
    name: String,
    range: (Position, Position),
    data_type: Option<String>,
}

impl Identifier {
    fn in_range(&self, pos: &Position) -> bool {
        // assumes identifier can not be multiline!
        let start = &self.range.0;
        let end = &self.range.1;
        start.line == pos.line && start.char <= pos.char && end.char >= pos.char
    }

    fn hover(&self) -> Option<Hover> {
        self.data_type.as_ref().map(|data_type| Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!("```kotlin\n{}: {}\n```", self.name, data_type,),
            }),
            range: None,
        })
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct FunctionBody {
    pub identifiers: Vec<Identifier>,
}

impl FunctionBody {
    fn populate_types(self, parameters: &[FunctionParameter]) -> FunctionBody {
        let mut typed_identifiers: Vec<Identifier> = Vec::new();
        for identifier in self.identifiers {
            if let Some(typed_id) = typed_identifiers.iter().find(|i| i.name == identifier.name) {
                typed_identifiers.push(Identifier {
                    name: identifier.name,
                    range: identifier.range,
                    data_type: typed_id.data_type.clone(),
                });
                continue;
            }

            if let Some(typed_id) = parameters.iter().find(|p| p.name == identifier.name) {
                typed_identifiers.push(Identifier {
                    name: identifier.name,
                    range: identifier.range,
                    data_type: Some(typed_id.type_identifier.clone()),
                });
                continue;
            }

            typed_identifiers.push(Identifier {
                name: identifier.name,
                range: identifier.range,
                data_type: None,
            });
        }

        FunctionBody {
            identifiers: typed_identifiers,
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Function {
    pub modifiers: Vec<FunctionModifier>,
    pub name: String,
    pub parameters: Vec<FunctionParameter>,
    pub return_type: Option<String>,
    pub body: Option<FunctionBody>,
}

impl Function {
    fn populate_types(self) -> Function {
        if let Some(body) = self.body {
            let body = body.populate_types(&self.parameters);

            Function {
                modifiers: self.modifiers,
                name: self.name,
                parameters: self.parameters,
                return_type: self.return_type,
                body: Some(body),
            }
        } else {
            Function {
                modifiers: self.modifiers,
                name: self.name,
                parameters: self.parameters,
                return_type: self.return_type,
                body: self.body,
            }
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct ClassBody {
    pub properties: Vec<Property>,
    pub functions: Vec<Function>,
}

impl ClassBody {
    fn populate_types(self) -> ClassBody {
        ClassBody {
            properties: self.properties,
            functions: self
                .functions
                .into_iter()
                .map(|f| f.populate_types())
                .collect(),
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct KotlinClass {
    pub name: String,
    pub modifiers: Vec<ClassModifier>,
    pub supertypes: Vec<String>,
    pub body: ClassBody,
}

impl KotlinClass {
    pub fn get_elem(&self, pos: &Position) -> Option<Hover> {
        for function in &self.body.functions {
            if let Some(body) = &function.body {
                for identifier in &body.identifiers {
                    if identifier.in_range(pos) {
                        return identifier.hover();
                    }
                }
            }
        }
        None
    }

    pub fn populate_types(self) -> KotlinClass {
        KotlinClass {
            name: self.name,
            modifiers: self.modifiers,
            supertypes: self.supertypes,
            body: self.body.populate_types(),
        }
    }
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
                    properties.push(get_property(&child, content)?);
                }

                if child.kind() == "function_declaration" {
                    functions.push(get_function(&child, content)?);
                }
            }
        }
    }

    Ok(ClassBody {
        properties,
        functions,
    })
}

fn get_property(node: &Node, content: &[u8]) -> Result<Property> {
    let mut modifiers: Vec<PropertyModifier> = Vec::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor.clone()) {
        if child.kind() == "modifiers" {
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
                    "inheritance_modifier" => modifiers.push(PropertyModifier::Inheritance(
                        child.utf8_text(content)?.to_string(),
                    )),
                    _ => bail!("unknown modifier {}", child.kind()),
                }
            }
        }

        if child.kind() == "variable_declaration" {
            let name = child
                .child(0)
                .context("no name found for variable declaration")?
                .utf8_text(content)?
                .to_string();
            let type_identifier = if let Some(type_node) = child.child(2) {
                Some(type_node.utf8_text(content)?.to_string())
            } else {
                None
            };

            return Ok(Property {
                modifiers,
                name,
                type_identifier,
            });
        }
    }

    bail!("no property found");
}

fn get_function(node: &Node, content: &[u8]) -> Result<Function> {
    let mut modifiers: Vec<FunctionModifier> = Vec::new();
    let mut parameters: Vec<FunctionParameter> = Vec::new();
    let mut name = None;
    let mut return_type = None;
    let mut body = None;
    let mut cursor = node.walk();
    for child in node.children(&mut cursor.clone()) {
        if child.kind() == "modifiers" {
            for child in child.children(&mut cursor) {
                match child.kind() {
                    "annotation" => modifiers.push(FunctionModifier::Annotation(
                        child.utf8_text(content)?.to_string(),
                    )),
                    "member_modifier" => modifiers.push(FunctionModifier::Member(
                        child.utf8_text(content)?.to_string(),
                    )),
                    "visibility_modifier" => modifiers.push(FunctionModifier::Visibility(
                        child.utf8_text(content)?.to_string(),
                    )),
                    "function_modifier" => modifiers.push(FunctionModifier::Function(
                        child.utf8_text(content)?.to_string(),
                    )),
                    "inheritance_modifier" => modifiers.push(FunctionModifier::Inheritance(
                        child.utf8_text(content)?.to_string(),
                    )),
                    _ => bail!("unknown modifier {}", child.kind()),
                }
            }
        }

        if child.kind() == "simple_identifier" {
            name = Some(child.utf8_text(content)?.to_string());
        }

        if child.kind() == "function_value_parameters" {
            for child in child.children(&mut cursor) {
                if child.kind() == "parameter" {
                    let name = child
                        .child(0)
                        .context("no parameter name found")?
                        .utf8_text(content)?
                        .to_string();

                    let type_identifier = child
                        .child(2)
                        .context("no type identifier found")?
                        .utf8_text(content)?
                        .to_string();

                    parameters.push(FunctionParameter {
                        name,
                        type_identifier,
                    })
                }
            }
        }

        if child.kind() == "user_type" {
            return_type = Some(child.utf8_text(content)?.to_string());
        }

        if child.kind() == "nullable_type" {
            return_type = Some(child.utf8_text(content)?.to_string());
        }

        if child.kind() == "function_body" {
            body = Some(get_function_body(&child, content)?);
        }
    }

    Ok(Function {
        modifiers,
        name: name.context("no name found for function")?,
        parameters,
        return_type,
        body,
    })
}

fn get_function_body(node: &Node, content: &[u8]) -> Result<FunctionBody> {
    let mut identifiers = Vec::new();
    let mut cursor = node.walk();
    loop {
        let node = cursor.node();

        if node.kind() == "simple_identifier" {
            let name = node.utf8_text(content)?.to_string();
            identifiers.push(Identifier {
                name,
                range: (
                    Position::new(node.start_position().row, node.start_position().column),
                    Position::new(node.end_position().row, node.end_position().column),
                ),
                data_type: None,
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
                return Ok(FunctionBody { identifiers });
            }
        }
    }
}

#[cfg(test)]
mod test {
    use tree_sitter::Parser;

    use crate::kotlin::{class::Function, KotlinFile, Position};

    use super::{FunctionBody, FunctionModifier, FunctionParameter, Identifier};

    #[test]
    fn function_parsing() {
        let expected = vec![
            Function {
                modifiers: vec![FunctionModifier::Inheritance("abstract".to_string())],
                name: "onLongClick".to_string(),
                parameters: vec![FunctionParameter {
                    name: "view".to_string(),
                    type_identifier: "View".to_string(),
                }],
                return_type: None,
                body: None,
            },
            Function {
                modifiers: vec![
                    FunctionModifier::Annotation("@Bar".to_string()),
                    FunctionModifier::Function("suspend".to_string()),
                    FunctionModifier::Visibility("private".to_string()),
                ],
                name: "concatenate".to_string(),
                parameters: vec![
                    FunctionParameter {
                        name: "str1".to_string(),
                        type_identifier: "String".to_string(),
                    },
                    FunctionParameter {
                        name: "str2".to_string(),
                        type_identifier: "String".to_string(),
                    },
                ],
                return_type: Some("String".to_string()),
                body: Some(FunctionBody {
                    identifiers: vec![
                        Identifier {
                            name: "str1".to_string(),
                            range: (Position::new(5, 15), Position::new(5, 19)),
                            data_type: Some("String".to_string()),
                        },
                        Identifier {
                            name: "str2".to_string(),
                            range: (Position::new(5, 22), Position::new(5, 26)),
                            data_type: Some("String".to_string()),
                        },
                    ],
                }),
            },
        ];

        let foo = include_bytes!("../../data/Foo.kt");
        let mut parser = Parser::new();
        parser.set_language(tree_sitter_kotlin::language()).unwrap();
        let tree = parser.parse(foo, None).unwrap();

        let file = KotlinFile::new(&tree, foo).unwrap();
        let file = file.populate_types();

        let body = &file.classes.get(0).unwrap().body;

        for (actual, expected) in body.functions.iter().zip(expected) {
            assert_eq!(*actual, expected);
        }
    }
}
