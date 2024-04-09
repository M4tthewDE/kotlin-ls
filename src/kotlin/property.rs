use anyhow::{bail, Context, Result};
use tree_sitter::Node;

use crate::kotlin::{
    expression::{Expression, EXPRESSIONS},
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
        Ok(PropertyDelegate {
            expression: Expression::new(
                &node.child(1).context(format!(
                    "[PropertyDelegate] no expression at {}",
                    node.start_position(),
                ))?,
                content,
            )?,
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
        let mut modifiers = Vec::new();
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
                "property_delegate" => delegate = Some(PropertyDelegate::new(&child, content)?),
                "getter" => getter = Some(Getter::new(&child, content)?),
                "setter" => setter = Some(Setter::new(&child, content)?),
                kind => {
                    if EXPRESSIONS.contains(&kind) {
                        expression = Some(Expression::new(&child, content)?)
                    } else {
                        bail!(
                            "[Property] unhandled child {} '{}' at {}",
                            child.kind(),
                            child.utf8_text(content)?,
                            child.start_position(),
                        )
                    }
                }
            }
        }

        // getter and setter can be both inside of property_declaration and outside!
        if let Some(next) = node.next_sibling() {
            match next.kind() {
                "getter" => getter = Some(Getter::new(&next, content)?),
                "setter" => setter = Some(Setter::new(&next, content)?),
                _ => {}
            }
        }

        Ok(Property {
            modifiers,
            variable_declaration: variable_declaration
                .context("[Property] no variable declaration found")?,
            extension_type,
            expression,
            mutability: mutability.context("[Property] no mutability modifier found")?,
            getter,
            setter,
            delegate,
        })
    }
}
