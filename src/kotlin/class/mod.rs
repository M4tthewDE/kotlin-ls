use anyhow::{bail, Context, Result};
use tower_lsp::lsp_types::Hover;
use tree_sitter::{Node, Tree};

use self::function::{Function, Property};

use super::Position;

mod function;

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum ClassModifier {
    Class(String),
    Visibility(String),
    Annotation(String),
    Inheritance(String),
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

#[cfg(test)]
mod test {
    use tree_sitter::Parser;

    use crate::kotlin::{class::Function, KotlinFile, Position};

    use super::function::{
        DataType, FunctionBody, FunctionModifier, FunctionParameter, Identifier,
    };

    #[test]
    fn class_parsing() {
        let expected = vec![
            Function {
                modifiers: vec![FunctionModifier::Inheritance("abstract".to_string())],
                name: "onLongClick".to_string(),
                parameters: vec![FunctionParameter {
                    name: "view".to_string(),
                    type_identifier: DataType("View".to_string()),
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
                        type_identifier: DataType("String".to_string()),
                    },
                    FunctionParameter {
                        name: "str2".to_string(),
                        type_identifier: DataType("String".to_string()),
                    },
                ],
                return_type: Some("String".to_string()),
                body: Some(FunctionBody {
                    identifiers: vec![
                        Identifier {
                            name: "str1".to_string(),
                            range: (Position::new(5, 15), Position::new(5, 19)),
                            data_type: Some(DataType("String".to_string())),
                        },
                        Identifier {
                            name: "str2".to_string(),
                            range: (Position::new(5, 22), Position::new(5, 26)),
                            data_type: Some(DataType("String".to_string())),
                        },
                    ],
                }),
            },
        ];

        let foo = include_bytes!("../../../data/Foo.kt");
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
