use anyhow::{bail, Context, Result};
use tower_lsp::lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind};
use tree_sitter::Node;

use crate::kotlin::Position;

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

impl Property {
    pub fn new(node: &Node, content: &[u8]) -> Result<Property> {
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
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum FunctionModifier {
    Annotation(String),
    Member(String),
    Visibility(String),
    Function(String),
    Inheritance(String),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct DataType(pub String);

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct FunctionParameter {
    pub name: String,
    pub type_identifier: DataType,
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Identifier {
    pub name: String,
    pub range: (Position, Position),
    pub data_type: Option<DataType>,
}

impl Identifier {
    pub fn in_range(&self, pos: &Position) -> bool {
        // assumes identifier can not be multiline!
        let start = &self.range.0;
        let end = &self.range.1;
        start.line == pos.line && start.char <= pos.char && end.char >= pos.char
    }

    pub fn hover(&self) -> Option<Hover> {
        self.data_type.as_ref().map(|data_type| Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!("```kotlin\n{}: {}\n```", self.name, data_type.0,),
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
    pub fn new(node: &Node, content: &[u8]) -> Result<FunctionBody> {
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
    pub fn new(node: &Node, content: &[u8]) -> Result<Function> {
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
                            type_identifier: DataType(type_identifier),
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
                body = Some(FunctionBody::new(&child, content)?);
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

    pub fn populate_types(self) -> Function {
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
