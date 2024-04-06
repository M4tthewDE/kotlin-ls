use anyhow::{bail, Context, Result};
use tree_sitter::Node;

use super::{
    argument::{self, Argument},
    lambda::AnnotatedLambda,
    literal::Literal,
    statement::{get_statements, Statement},
    types::Type,
};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Expression {
    Call {
        identifier: Option<String>,
        call_suffix: CallSuffix,
        expression: Box<Option<Expression>>,
    },
    Navigation {
        navigation_suffix: NavigationSuffix,
        expression: Box<Option<Expression>>,
    },
    If {
        expression: Box<Expression>,
        body: ControlStructureBody,
    },
    Equality {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Disjunction {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Identifier {
        identifier: String,
    },
    Infix {
        left: Box<Expression>,
        middle: String,
        right: Box<Expression>,
    },
    As {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Literal(Literal),
    When {
        subject: WhenSubject,
        entries: Vec<WhenEntry>,
    },
    CheckIn {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Elvis {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Type(Type),
}

impl Expression {
    pub fn new(node: &Node, content: &[u8]) -> Result<Expression> {
        match node.kind() {
            "call_expression" => call_expression(node, content),
            "navigation_expression" => navigation_expression(node, content),
            "if_expression" => if_expression(node, content),
            "disjunction_expression" => disjunction_expression(node, content),
            "equality_expression" => equality_expression(node, content),
            "simple_identifier" => identifier_expression(node, content),
            "infix_expression" => infix_expression(node, content),
            "as_expression" => as_expression(node, content),
            "elvis_expression" => elvis_expression(node, content),
            "check_expression" => check_expression(node, content),
            "boolean_literal" | "string_literal" | "integer_literal" | "null" => {
                literal_expression(node, content)
            }
            "when_expression" => when_expression(node, content),
            "user_type" => Ok(Expression::Type(Type::new(node, content)?)),
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
    let mut identifier = None;
    let mut call_suffix = None;
    let mut expression = None;
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "simple_identifier" => identifier = Some(child.utf8_text(content)?.to_string()),
            "call_suffix" => call_suffix = Some(CallSuffix::new(&child, content)?),
            "navigation_expression" | "call_expression" => {
                expression = Some(Expression::new(&child, content)?)
            }
            _ => {
                bail!(
                    "[Expression::Call] unhandled child {} '{}' at {}",
                    child.kind(),
                    child.utf8_text(content)?,
                    child.start_position(),
                )
            }
        }
    }

    Ok(Expression::Call {
        identifier,
        call_suffix: call_suffix.context("no call suffix found")?,
        expression: Box::new(expression),
    })
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct NavigationSuffix {
    identifier: String,
}

impl NavigationSuffix {
    pub fn new(node: &Node, content: &[u8]) -> Result<NavigationSuffix> {
        let mut identifier = None;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "." => {}
                "simple_identifier" => identifier = Some(child.utf8_text(content)?.to_string()),
                _ => {
                    bail!(
                        "[NavigationSuffix] unhandled child {} '{}' at {}",
                        child.kind(),
                        child.utf8_text(content)?,
                        child.start_position(),
                    )
                }
            }
        }

        Ok(NavigationSuffix {
            identifier: identifier.context("no identifier found")?,
        })
    }
}

fn navigation_expression(node: &Node, content: &[u8]) -> Result<Expression> {
    let mut navigation_suffix = None;
    let mut expression = None;
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "call_expression"
            | "navigation_expression"
            | "simple_identifier"
            | "integer_literal" => expression = Some(Expression::new(&child, content)?),
            "navigation_suffix" => {
                navigation_suffix = Some(NavigationSuffix::new(&child, content)?)
            }
            _ => {
                bail!(
                    "[Expression::Navigation] unhandled child {} '{}' at {}",
                    child.kind(),
                    child.utf8_text(content)?,
                    child.start_position(),
                )
            }
        }
    }

    Ok(Expression::Navigation {
        navigation_suffix: navigation_suffix.context("no call suffix found")?,
        expression: Box::new(expression),
    })
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct ControlStructureBody {
    statements: Vec<Statement>,
}

impl ControlStructureBody {
    pub fn new(node: &Node, content: &[u8]) -> Result<ControlStructureBody> {
        let mut statements = None;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "{" | "}" => {}
                "statements" => statements = Some(get_statements(&child, content)?),
                // FIXME: this feels like a hack, maybe directly create a statment instead?
                // we know at this point that there is only one
                "call_expression" => statements = Some(get_statements(node, content)?),
                "null" => {
                    statements = Some(vec![Statement::Expression(Expression::new(
                        &child, content,
                    )?)])
                }
                _ => {
                    bail!(
                        "[ControlStructureBody] unhandled child {} '{}' at {}",
                        child.kind(),
                        child.utf8_text(content)?,
                        child.start_position(),
                    )
                }
            }
        }

        Ok(ControlStructureBody {
            statements: statements.context(format!(
                "[ControlStructureBody] no statements found at {}",
                node.start_position()
            ))?,
        })
    }
}

fn if_expression(node: &Node, content: &[u8]) -> Result<Expression> {
    let mut expression = None;
    let mut body = None;
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "if" | "(" | ")" => {}
            "equality_expression" | "disjunction_expression" => {
                expression = Some(Expression::new(&child, content)?)
            }
            "control_structure_body" => body = Some(ControlStructureBody::new(&child, content)?),
            _ => {
                bail!(
                    "[Expression::If] unhandled child {} '{}' at {}",
                    child.kind(),
                    child.utf8_text(content)?,
                    child.start_position(),
                )
            }
        }
    }

    Ok(Expression::If {
        expression: Box::new(expression.context("[Expression::If] no expression found")?),
        body: body.context("[Expression::If] no control structure body found")?,
    })
}

fn equality_expression(node: &Node, content: &[u8]) -> Result<Expression> {
    let mut left = None;
    let mut right = None;
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        let expression = match child.kind() {
            "==" | "!=" => None,
            "simple_identifier" | "null" | "navigation_expression" => {
                Some(Expression::new(&child, content)?)
            }
            _ => {
                bail!(
                    "[Expression::Equality] unhandled child {} '{}' at {}",
                    child.kind(),
                    child.utf8_text(content)?,
                    child.start_position(),
                )
            }
        };

        if expression.is_some() {
            if left.is_none() {
                left = expression;
            } else {
                right = expression;
            }
        }
    }

    Ok(Expression::Equality {
        left: Box::new(left.context("[Expression::Equality] no left eexpression found")?),
        right: Box::new(right.context("[Expression::Equality] no right eexpression found")?),
    })
}

fn disjunction_expression(node: &Node, content: &[u8]) -> Result<Expression> {
    let mut left = None;
    let mut right = None;
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        let expression = match child.kind() {
            "||" => None,
            "simple_identifier" | "equality_expression" => Some(Expression::new(&child, content)?),
            _ => {
                bail!(
                    "[Expression::Disjunction] unhandled child {} '{}' at {}",
                    child.kind(),
                    child.utf8_text(content)?,
                    child.start_position(),
                )
            }
        };

        if expression.is_some() {
            if left.is_none() {
                left = expression;
            } else {
                right = expression;
            }
        }
    }

    Ok(Expression::Disjunction {
        left: Box::new(left.context("[Expression::Disjunction] no left eexpression found")?),
        right: Box::new(right.context("[Expression::Disjunction] no right eexpression found")?),
    })
}

fn identifier_expression(node: &Node, content: &[u8]) -> Result<Expression> {
    if node.kind() != "simple_identifier" {
        bail!(
            "[Expression::Identifier]  invalid node {} '{}' at {}",
            node.kind(),
            node.utf8_text(content)?,
            node.start_position(),
        );
    }

    Ok(Expression::Identifier {
        identifier: node.utf8_text(content)?.to_string(),
    })
}

fn literal_expression(node: &Node, content: &[u8]) -> Result<Expression> {
    Ok(Expression::Literal(Literal::new(node, content)?))
}

fn infix_expression(node: &Node, content: &[u8]) -> Result<Expression> {
    let left_node = node
        .child(0)
        .context(format!("too little children at {}", node.start_position()))?;
    let left: Result<Expression> = match left_node.kind() {
        "simple_identifier" => Ok(Expression::new(&left_node, content)?),
        _ => {
            bail!(
                "[Expression::Infix] unhandled child {} '{}' at {}",
                node.kind(),
                node.utf8_text(content)?,
                node.start_position(),
            )
        }
    };

    let middle_node = node
        .child(1)
        .context(format!("too little children at {}", node.start_position()))?;

    if middle_node.kind() != "simple_identifier" {
        bail!(
            "[Expression::Infix] incompatible middle node {} '{}' at {}",
            node.kind(),
            node.utf8_text(content)?,
            node.start_position(),
        );
    }

    let middle = middle_node.utf8_text(content)?.to_string();

    let right_node = node.child(2).context(format!(
        "[Expression::Infix] too little children at {}",
        node.start_position()
    ))?;
    let right: Result<Expression> = match right_node.kind() {
        "simple_identifier" => Ok(Expression::new(&right_node, content)?),
        _ => {
            bail!(
                "[Expression::Infix] unhandled child {} '{}' at {}",
                node.kind(),
                node.utf8_text(content)?,
                node.start_position(),
            )
        }
    };

    Ok(Expression::Infix {
        left: Box::new(left?),
        middle,
        right: Box::new(right.context("[Expression::Infix] no right expression found")?),
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
    let mut left = None;
    let mut right = None;
    let mut check_type = None;
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "in" => check_type = Some("in".to_string()),
            "simple_identifier" => {
                if left.is_none() {
                    left = Some(Expression::new(&child, content)?)
                } else if let Some(ref c) = check_type {
                    if c == "in" {
                        right = Some(Expression::new(&child, content)?)
                    } else {
                        bail!(
                            "[Expression::Check] invalid check type {} for {} at {}",
                            c,
                            child.kind(),
                            child.start_position(),
                        )
                    }
                } else {
                    bail!(
                        "[Expression::Check] check type has to be known at {}",
                        child.start_position()
                    )
                }
            }
            _ => {
                bail!(
                    "[Expression::Check] unhandled child {} '{}' at {}",
                    child.kind(),
                    child.utf8_text(content)?,
                    child.start_position(),
                )
            }
        }
    }

    Ok(Expression::CheckIn {
        left: Box::new(left.context(format!(
            "[Expression::Check] no left side at {}",
            node.start_position()
        ))?),
        right: Box::new(right.context(format!(
            "[Expression::Check] no right side at {}",
            node.start_position()
        ))?),
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
}

impl WhenCondition {
    fn new(node: &Node, content: &[u8]) -> Result<WhenCondition> {
        return if let Some(child) = node.child(0) {
            match child.kind() {
                "simple_identifier" => {
                    Ok(WhenCondition::Expression(Expression::new(&child, content)?))
                }
                _ => {
                    bail!(
                        "[WhenCondition] unhandled child {} '{}' at {}",
                        child.kind(),
                        child.utf8_text(content)?,
                        child.start_position(),
                    )
                }
            }
        } else {
            bail!(
                "[WhenCondition] unhandled node {} '{}' at {}",
                node.kind(),
                node.utf8_text(content)?,
                node.start_position(),
            )
        };
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
            "when" | "{" | "}" => {}
            "when_subject" => subject = Some(WhenSubject::new(&child, content)?),
            "when_entry" => entries.push(WhenEntry::new(&child, content)?),
            _ => {
                bail!(
                    "[Expression::When] unhandled child {} '{}' at {}",
                    child.kind(),
                    child.utf8_text(content)?,
                    child.start_position(),
                )
            }
        }
    }

    Ok(Expression::When {
        subject: subject.context(format!(
            "[Expression::When] no expression at {}",
            node.start_position()
        ))?,
        entries,
    })
}
