use anyhow::{bail, Context, Result};
use tree_sitter::Node;

use crate::kotlin::{
    expression::Expression,
    getter::{Getter, Setter},
    Type,
};

use super::{modifier::Modifier, variable_declaration::VariableDeclaration};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum PropertyMutability {
    Var,
    Val,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Property {
    pub modifiers: Vec<Modifier>,
    pub variable_declaration: VariableDeclaration,
    pub extension_type: Option<Type>,
    pub mutability: PropertyMutability,
    pub expression: Option<Expression>,
    pub getter: Option<Getter>,
    pub setter: Option<Setter>,
}

impl Property {
    pub fn new(node: &Node, content: &[u8]) -> Result<Property> {
        let mut modifiers: Vec<Modifier> = Vec::new();
        let mut variable_declaration = None;
        let mut mutability = None;
        let mut extension_type = None;
        let mut expression = None;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor.clone()) {
            match child.kind() {
                "modifiers" => {
                    for child in child.children(&mut cursor) {
                        modifiers.push(Modifier::new(&child, content)?);
                    }
                }
                "var" => mutability = Some(PropertyMutability::Var),
                "val" => mutability = Some(PropertyMutability::Val),
                "user_type" => {
                    extension_type = Some(Type::NonNullable(child.utf8_text(content)?.to_string()))
                }
                "nullable_type" => {
                    extension_type = Some(Type::Nullable(child.utf8_text(content)?.to_string()))
                }
                "variable_declaration" => {
                    variable_declaration = Some(VariableDeclaration::new(&child, content)?)
                }
                "." | "=" => {}
                "call_expression" | "when_expression" | "string_literal" | "integer_literal"
                | "check_expression" => expression = Some(Expression::new(&child, content)?),
                _ => {
                    bail!(
                        "[Property] unhandled child {} '{}' at {}",
                        child.kind(),
                        child.utf8_text(content)?,
                        child.start_position(),
                    )
                }
            }
        }

        let mut getter = None;
        let mut setter = None;

        if let Some(next) = node.next_sibling() {
            match next.kind() {
                "getter" => getter = Some(Getter::new(&next, content)?),
                "setter" => setter = Some(Setter::new(&next, content)?),
                _ => {}
            }
        }

        Ok(Property {
            modifiers,
            variable_declaration: variable_declaration.context("no variable declaration found")?,
            extension_type,
            expression,
            mutability: mutability.context("no mutability modifier found")?,
            getter,
            setter,
        })
    }
}
