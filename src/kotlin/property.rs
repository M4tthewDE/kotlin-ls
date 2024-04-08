use anyhow::{bail, Context, Result};
use tree_sitter::Node;

use crate::kotlin::{
    expression::Expression,
    getter::{Getter, Setter},
    types::Type,
};

use super::{modifier::Modifier, variable_declaration::VariableDeclaration};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum PropertyMutability {
    Var,
    Val,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct PropertyDelegate {
    expression: Expression,
}

impl PropertyDelegate {
    pub fn new(node: &Node, content: &[u8]) -> Result<PropertyDelegate> {
        let mut expression = None;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "by" => {}
                "call_expression" => expression = Some(Expression::new(&child, content)?),
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

        Ok(PropertyDelegate {
            expression: expression.context(format!(
                "[Property] no expression found at {}",
                node.start_position()
            ))?,
        })
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Property {
    pub modifiers: Vec<Modifier>,
    pub variable_declaration: VariableDeclaration,
    pub extension_type: Option<Type>,
    pub mutability: PropertyMutability,
    pub expression: Option<Expression>,
    pub delegate: Option<PropertyDelegate>,
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
        let mut getter = None;
        let mut setter = None;
        let mut delegate = None;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor.clone()) {
            match child.kind() {
                "." | "=" => {}
                "modifiers" => {
                    for child in child.children(&mut cursor) {
                        modifiers.push(Modifier::new(&child, content)?);
                    }
                }
                "var" => mutability = Some(PropertyMutability::Var),
                "val" => mutability = Some(PropertyMutability::Val),
                "user_type" | "nullable_type" => extension_type = Some(Type::new(&child, content)?),
                "variable_declaration" => {
                    variable_declaration = Some(VariableDeclaration::new(&child, content)?)
                }
                "call_expression"
                | "when_expression"
                | "string_literal"
                | "integer_literal"
                | "boolean_literal"
                | "object_literal"
                | "check_expression"
                | "null"
                | "elvis_expression"
                | "navigation_expression" => expression = Some(Expression::new(&child, content)?),
                "property_delegate" => delegate = Some(PropertyDelegate::new(&child, content)?),
                "getter" => getter = Some(Getter::new(&child, content)?),
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
            delegate,
        })
    }
}
