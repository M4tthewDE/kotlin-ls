use anyhow::{bail, Context, Result};
use tree_sitter::Node;

use super::{
    argument::{self, ValueArgument},
    lambda::AnnotatedLambda,
    literal::Literal,
    statement::{get_statements, Statement},
};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Expression {
    Call {
        identifier: Option<String>,
        call_suffix: CallSuffix,
        expression: Box<Option<Expression>>,
    },
    Navigation {
        identifier: Option<String>,
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
    Literal(Literal),
    When {
        subject: WhenSubject,
        entries: Vec<WhenEntry>,
    },
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
            "boolean_literal" => literal_expression(node, content),
            "when_expression" => when_expression(node, content),
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
    arguments: Option<Vec<ValueArgument>>,
    annotated_lambda: Option<AnnotatedLambda>,
}

impl CallSuffix {
    pub fn new(node: &Node, content: &[u8]) -> Result<CallSuffix> {
        let mut arguments = None;
        let mut annotated_lambda = None;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "value_arguments" => arguments = Some(argument::get_arguments(&child, content)?),
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
    let mut identifier = None;
    let mut navigation_suffix = None;
    let mut expression = None;
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "simple_identifier" => identifier = Some(child.utf8_text(content)?.to_string()),
            "call_expression" | "navigation_expression" => {
                expression = Some(Expression::new(&child, content)?)
            }
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
        identifier,
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
                    statements = Some(vec![Statement::Expression(Expression::Literal(
                        Literal::Null,
                    ))])
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
            "null" => Some(Expression::Literal(Literal::Null)),
            "simple_identifier" => Some(Expression::new(&child, content)?),
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

    let right_node = node
        .child(0)
        .context(format!("too little children at {}", node.start_position()))?;
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
        right: Box::new(right.context("[Expression::Equality] no right expression found")?),
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
