use anyhow::{bail, Context, Result};
use tree_sitter::Node;

use super::{
    argument::{self, Argument},
    label::Label,
    lambda::AnnotatedLambda,
    literal::Literal,
    statement::{self, Statement},
    types::Type,
};

mod jump;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum EqualityOperator {
    ReferentialEquality,
    StructuralEquality,
    ReferentialInequality,
    StructuralInequality,
}

impl EqualityOperator {
    fn new(node: &Node, content: &[u8]) -> Result<EqualityOperator> {
        Ok(match node.utf8_text(content)? {
            "==" => EqualityOperator::StructuralEquality,
            "===" => EqualityOperator::ReferentialEquality,
            "!=" => EqualityOperator::StructuralInequality,
            "!==" => EqualityOperator::ReferentialInequality,
            _ => bail!(
                "[EqualityOperator] Invalid equality operator at {}",
                node.start_position()
            ),
        })
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Expression {
    Call {
        expression: Box<Expression>,
        call_suffix: CallSuffix,
    },
    Navigation {
        expression: Box<Expression>,
        navigation_suffix: NavigationSuffix,
    },
    If {
        expression: Box<Expression>,
        body: ControlStructureBody,
    },
    Equality {
        left: Box<Expression>,
        operator: EqualityOperator,
        right: Box<Expression>,
    },
    Disjunction {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Conjunction {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Identifier {
        identifier: String,
    },
    Infix {
        left: Box<Expression>,
        identifier: String,
        right: Box<Expression>,
    },
    As {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Literal(Literal),
    When {
        subject: Option<WhenSubject>,
        entries: Vec<WhenEntry>,
    },
    CheckIn {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    CheckAs {
        left: Box<Expression>,
        right: Type,
    },
    Elvis {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Type(Type),
    JumpThrow(Box<Expression>),
    JumpReturn(Option<Label>, Box<Option<Expression>>),
    JumpContinue(Option<Label>),
    JumpBreak(Option<Label>),
    DirectlyAssignable(Box<Expression>),
    CallableReference {
        left: Option<String>,
        right: String,
    },
}

impl Expression {
    pub fn new(node: &Node, content: &[u8]) -> Result<Expression> {
        match node.kind() {
            "call_expression" => call_expression(node, content),
            "navigation_expression" => navigation_expression(node, content),
            "if_expression" => if_expression(node, content),
            "disjunction_expression" => disjunction_expression(node, content),
            "conjunction_expression" => conjunction_expression(node, content),
            "equality_expression" => equality_expression(node, content),
            "simple_identifier" => Ok(Expression::Identifier {
                identifier: node.utf8_text(content)?.to_string(),
            }),
            "infix_expression" => infix_expression(node, content),
            "as_expression" => as_expression(node, content),
            "elvis_expression" => elvis_expression(node, content),
            "check_expression" => check_expression(node, content),
            "callable_reference" => callable_reference(node, content),
            "boolean_literal" | "string_literal" | "integer_literal" | "null" => {
                Ok(Expression::Literal(Literal::new(node, content)?))
            }
            "when_expression" => when_expression(node, content),
            "user_type" => Ok(Expression::Type(Type::new(node, content)?)),
            "jump_expression" => jump::expression(node, content),
            "directly_assignable_expression" => {
                Ok(Expression::DirectlyAssignable(Box::new(Expression::new(
                    &node.child(0).context(format!(
                        "[Expression::DirectlyAssignable] no child at {}",
                        node.start_position()
                    ))?,
                    content,
                )?)))
            }
            _ => {
                bail!(
                    "[Expression] unhandled child {} '{}' at {}",
                    node.kind(),
                    node.utf8_text(content)?,
                    node.start_position(),
                )
            }
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct CallSuffix {
    arguments: Option<Vec<Argument>>,
    annotated_lambda: Option<AnnotatedLambda>,
}

impl CallSuffix {
    pub fn new(node: &Node, content: &[u8]) -> Result<CallSuffix> {
        let mut arguments = None;
        let mut annotated_lambda = None;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "value_arguments" => {
                    arguments = Some(argument::get_value_arguments(&child, content)?)
                }
                // NOTE: this is plurarl, but there is only one type_argument with multiple type
                // projections!
                "type_arguments" => {
                    arguments = Some(vec![argument::get_type_argument(&child, content)?])
                }
                "annotated_lambda" => {
                    annotated_lambda = Some(AnnotatedLambda::new(&child, content)?)
                }
                _ => {
                    bail!(
                        "[CallSuffix] unhandled child {} '{}' at {}",
                        child.kind(),
                        child.utf8_text(content)?,
                        child.start_position(),
                    )
                }
            }
        }

        Ok(CallSuffix {
            arguments,
            annotated_lambda,
        })
    }
}

fn call_expression(node: &Node, content: &[u8]) -> Result<Expression> {
    Ok(Expression::Call {
        expression: Box::new(Expression::new(
            &node.child(0).context(format!(
                "[Expression::Call] no expression found at {}",
                node.start_position()
            ))?,
            content,
        )?),
        call_suffix: CallSuffix::new(
            &node.child(1).context(format!(
                "[Expression::Call] no expression found at {}",
                node.start_position()
            ))?,
            content,
        )?,
    })
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct NavigationSuffix {
    identifier: String,
}

impl NavigationSuffix {
    pub fn new(node: &Node, content: &[u8]) -> Result<NavigationSuffix> {
        let child = node.child(1).context(format!(
            "[NavigationSuffix] no navigation_suffix found at {}",
            node.start_position()
        ))?;

        Ok(NavigationSuffix {
            identifier: match child.kind() {
                "simple_identifier" => child.utf8_text(content)?.to_string(),
                _ => {
                    bail!(
                        "[NavigationSuffix] unhandled child {} '{}' at {}",
                        child.kind(),
                        child.utf8_text(content)?,
                        child.start_position(),
                    )
                }
            },
        })
    }
}

fn navigation_expression(node: &Node, content: &[u8]) -> Result<Expression> {
    Ok(Expression::Navigation {
        expression: Box::new(Expression::new(
            &node.child(0).context(format!(
                "[Expression::Call] no expression found at {}",
                node.start_position()
            ))?,
            content,
        )?),
        navigation_suffix: NavigationSuffix::new(
            &node.child(1).context(format!(
                "[Expression::Call] no expression found at {}",
                node.start_position()
            ))?,
            content,
        )?,
    })
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct ControlStructureBody {
    statements: Vec<Statement>,
}

impl ControlStructureBody {
    pub fn new(node: &Node, content: &[u8]) -> Result<ControlStructureBody> {
        let child = node.child(0).context(format!(
            "[ControlStructureBody] no child at {}",
            node.start_position()
        ))?;

        match child.kind() {
            "{" => Ok(ControlStructureBody {
                statements: statement::get_statements(
                    &node.child(1).context(format!(
                        "[ControlStructureBody] no child at {}",
                        node.start_position()
                    ))?,
                    content,
                )
                .unwrap_or_default(),
            }),
            _ => Ok(ControlStructureBody {
                statements: statement::get_statements(&child, content).unwrap_or_default(),
            }),
        }
    }
}

fn if_expression(node: &Node, content: &[u8]) -> Result<Expression> {
    Ok(Expression::If {
        expression: Box::new(Expression::new(
            &node.child(2).context(format!(
                "[Expression::If] no expression found at {}",
                node.start_position()
            ))?,
            content,
        )?),
        body: ControlStructureBody::new(
            &node.child(4).context(format!(
                "[Expression::If] no control structure body found at {}",
                node.start_position()
            ))?,
            content,
        )?,
    })
}

fn equality_expression(node: &Node, content: &[u8]) -> Result<Expression> {
    Ok(Expression::Equality {
        left: Box::new(Expression::new(
            &node.child(0).context(format!(
                "[Expression::Equality] no expression found at {}",
                node.start_position()
            ))?,
            content,
        )?),
        operator: EqualityOperator::new(
            &node.child(1).context(format!(
                "[Expression::Equality] no operator found at {}",
                node.start_position()
            ))?,
            content,
        )?,
        right: Box::new(Expression::new(
            &node.child(2).context(format!(
                "[Expression::Equality] no expression found at {}",
                node.start_position()
            ))?,
            content,
        )?),
    })
}

fn disjunction_expression(node: &Node, content: &[u8]) -> Result<Expression> {
    Ok(Expression::Disjunction {
        left: Box::new(Expression::new(
            &node.child(0).context(format!(
                "[Expression::Disjunction] no expression found at {}",
                node.start_position()
            ))?,
            content,
        )?),
        right: Box::new(Expression::new(
            &node.child(2).context(format!(
                "[Expression::Disjunction] no expression found at {}",
                node.start_position()
            ))?,
            content,
        )?),
    })
}

fn callable_reference(node: &Node, content: &[u8]) -> Result<Expression> {
    let first_node = &node.child(0).context(format!(
        "[Expression::CallableReference] too little children at {}",
        node.start_position()
    ))?;

    let (left, right) = match first_node.kind() {
        "::" => (
            None,
            node.child(1)
                .context(format!(
                    "[Expression::CallableReference] too little children at {}",
                    node.start_position()
                ))?
                .utf8_text(content)?
                .to_string(),
        ),
        _ => (
            Some(first_node.utf8_text(content)?.to_string()),
            node.child(1)
                .context(format!(
                    "[Expression::CallableReference] too little children at {}",
                    node.start_position()
                ))?
                .utf8_text(content)?
                .to_string(),
        ),
    };
    Ok(Expression::CallableReference { left, right })
}

fn conjunction_expression(node: &Node, content: &[u8]) -> Result<Expression> {
    Ok(Expression::Conjunction {
        left: Box::new(Expression::new(
            &node.child(0).context(format!(
                "[Expression::Conjunction] no expression found at {}",
                node.start_position()
            ))?,
            content,
        )?),
        right: Box::new(Expression::new(
            &node.child(2).context(format!(
                "[Expression::Conjunction] no expression found at {}",
                node.start_position()
            ))?,
            content,
        )?),
    })
}

fn infix_expression(node: &Node, content: &[u8]) -> Result<Expression> {
    Ok(Expression::Infix {
        left: Box::new(Expression::new(
            &node.child(0).context(format!(
                "[Expression::Infix] no expression found at {}",
                node.start_position()
            ))?,
            content,
        )?),
        identifier: node
            .child(1)
            .context(format!(
                "[Expression::Infix] no middle found at {}",
                node.start_position()
            ))?
            .utf8_text(content)?
            .to_string(),
        right: Box::new(Expression::new(
            &node.child(2).context(format!(
                "[Expression::Infix] no expression found at {}",
                node.start_position()
            ))?,
            content,
        )?),
    })
}

fn as_expression(node: &Node, content: &[u8]) -> Result<Expression> {
    Ok(Expression::As {
        left: Box::new(Expression::new(
            &node.child(0).context(format!(
                "[Expression::As] too little children at {}",
                node.start_position()
            ))?,
            content,
        )?),
        right: Box::new(Expression::new(
            &node.child(2).context(format!(
                "[Expression::As] too little children at {}",
                node.start_position()
            ))?,
            content,
        )?),
    })
}

fn elvis_expression(node: &Node, content: &[u8]) -> Result<Expression> {
    Ok(Expression::Elvis {
        left: Box::new(Expression::new(
            &node.child(0).context(format!(
                "[Expression::Elvis] too little children at {}",
                node.start_position()
            ))?,
            content,
        )?),
        right: Box::new(Expression::new(
            &node.child(2).context(format!(
                "[Expression::Elvis] too little children at {}",
                node.start_position()
            ))?,
            content,
        )?),
    })
}

fn check_expression(node: &Node, content: &[u8]) -> Result<Expression> {
    let operator_node = &node.child(1).context(format!(
        "[Expression::Check] too little children at {}",
        node.start_position()
    ))?;

    Ok(match operator_node.kind() {
        "in" => Expression::CheckIn {
            left: Box::new(Expression::new(
                &node.child(0).context(format!(
                    "[Expression::Check] too little children at {}",
                    node.start_position()
                ))?,
                content,
            )?),
            right: Box::new(Expression::new(
                &node.child(2).context(format!(
                    "[Expression::Check] too little children at {}",
                    node.start_position()
                ))?,
                content,
            )?),
        },
        "as" => Expression::CheckAs {
            left: Box::new(Expression::new(
                &node.child(0).context(format!(
                    "[Expression::Check] too little children at {}",
                    node.start_position()
                ))?,
                content,
            )?),
            right: Type::new(
                &node.child(2).context(format!(
                    "[Expression::Check] too little children at {}",
                    node.start_position()
                ))?,
                content,
            )?,
        },
        op => {
            bail!(
                "[Expression::Check] unhandled operator {} at {}",
                op,
                node.start_position(),
            )
        }
    })
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct WhenSubject {
    expression: Box<Expression>,
}

impl WhenSubject {
    fn new(node: &Node, content: &[u8]) -> Result<WhenSubject> {
        let mut expression = None;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "(" | ")" => {}
                "simple_identifier" => expression = Some(Expression::new(&child, content)?),
                _ => {
                    bail!(
                        "[WhenSubject] unhandled child {} '{}' at {}",
                        child.kind(),
                        child.utf8_text(content)?,
                        child.start_position(),
                    )
                }
            }
        }

        Ok(WhenSubject {
            expression: Box::new(expression.context(format!(
                "[WhenSubject] no expression at {}",
                node.start_position()
            ))?),
        })
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum WhenCondition {
    Expression(Expression),
    RangeTest(Expression),
    TypeTest(Type),
}

impl WhenCondition {
    fn new(node: &Node, content: &[u8]) -> Result<WhenCondition> {
        let child = node.child(0).context(format!(
            "[WhenCondition] no child 0 at {}",
            node.start_position(),
        ))?;

        Ok(match child.kind() {
            "in" => WhenCondition::RangeTest(Expression::new(
                &node.child(1).context(format!(
                    "[WhenCondition] no child 1 at {}",
                    node.start_position(),
                ))?,
                content,
            )?),
            "as" => WhenCondition::TypeTest(Type::new(
                &node.child(1).context(format!(
                    "[WhenCondition] no child 1 at {}",
                    node.start_position(),
                ))?,
                content,
            )?),
            _ => WhenCondition::Expression(Expression::new(&child, content)?),
        })
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct WhenEntry {
    // condition is empty for "else" case
    condition: Option<WhenCondition>,
    body: ControlStructureBody,
}

impl WhenEntry {
    fn new(node: &Node, content: &[u8]) -> Result<WhenEntry> {
        let mut condition = None;
        let mut body = None;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "->" | "else" => {}
                "when_condition" => condition = Some(WhenCondition::new(&child, content)?),
                "control_structure_body" => {
                    body = Some(ControlStructureBody::new(&child, content)?)
                }
                _ => {
                    bail!(
                        "[WhenEntry] unhandled child {} '{}' at {}",
                        child.kind(),
                        child.utf8_text(content)?,
                        child.start_position(),
                    )
                }
            }
        }

        Ok(WhenEntry {
            condition,
            body: body.context(format!("[WhenEntry] no body at {}", node.start_position()))?,
        })
    }
}

fn when_expression(node: &Node, content: &[u8]) -> Result<Expression> {
    let mut subject = None;
    let mut entries = Vec::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "when_subject" => subject = Some(WhenSubject::new(&child, content)?),
            "when_entry" => entries.push(WhenEntry::new(&child, content)?),
            _ => {}
        }
    }

    Ok(Expression::When { subject, entries })
}
