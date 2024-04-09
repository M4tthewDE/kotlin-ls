use anyhow::{bail, Context, Result};
use tree_sitter::Node;

use self::r#try::{CatchBlock, FinallyBlock};

use super::{
    argument::{self, Argument},
    label::Label,
    lambda::AnnotatedLambda,
    literal::Literal,
    statement::{self, Statement},
    types::Type,
};

mod jump;
mod r#try;

pub const EXPRESSIONS: [&str; 40] = [
    // unary
    "postfix_expression",
    "call_expression",
    "indexing_expression",
    "navigation_expression",
    "prefix_expression",
    "as_expression",
    "spread_expression",
    // binary
    "multiplicative_expression",
    "additive_expression",
    "range_expression",
    "infix_expression",
    "elvis_expression",
    "check_expression",
    "comparison_expression",
    "equality_expression",
    "conjunction_expression",
    "disjunction_expression",
    // primary
    "parenthesized_expression",
    "simple_identifier",
    "boolean_literal",
    "integer_literal",
    "hex_literal",
    "bin_literal",
    "character_literal",
    "real_literal",
    "null",
    "long_literal",
    "unsigned_literal",
    "string_literal",
    "callable_reference",
    "lambda_literal",
    "anonymous_function",
    "object_literal",
    "collection_literal",
    "this_expression",
    "super_expression",
    "if_expression",
    "when_expression",
    "try_expression",
    "jump_expression",
];

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum MultiplicativeOperator {
    Mul,
    Div,
    Mod,
}

impl MultiplicativeOperator {
    fn new(node: &Node, content: &[u8]) -> Result<MultiplicativeOperator> {
        Ok(match node.utf8_text(content)? {
            "*" => MultiplicativeOperator::Mul,
            "/" => MultiplicativeOperator::Div,
            "%" => MultiplicativeOperator::Mod,
            _ => bail!(
                "[MultiplicativeOperator] Invalid multiplicative operator at {}",
                node.start_position()
            ),
        })
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum PrefixUnaryOperator {
    Increment,
    Decrement,
    Minus,
    Plus,
    Negation,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum PostfixUnaryOperator {
    Increment,
    Decrement,
    NullAssertion,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum ComparisonOperator {
    Less,
    Greater,
    LessEquals,
    GreaterEquals,
}

impl ComparisonOperator {
    fn new(node: &Node, content: &[u8]) -> Result<ComparisonOperator> {
        Ok(match node.utf8_text(content)? {
            "<" => ComparisonOperator::Less,
            ">" => ComparisonOperator::Greater,
            "<=" => ComparisonOperator::LessEquals,
            ">=" => ComparisonOperator::GreaterEquals,
            _ => bail!(
                "[ComparisonOperator] Invalid comparison operator at {}",
                node.start_position()
            ),
        })
    }
}

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
    Multiplicative {
        left: Box<Expression>,
        operator: MultiplicativeOperator,
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
    Additive {
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
    CheckIs {
        left: Box<Expression>,
        right: Type,
    },
    CheckNotIn {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    CheckNotIs {
        left: Box<Expression>,
        right: Type,
    },
    Elvis {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Range {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Type(Type),
    JumpThrow(Box<Expression>),
    JumpReturn(Option<Label>, Option<Box<Expression>>),
    JumpContinue(Option<Label>),
    JumpBreak(Option<Label>),
    DirectlyAssignable(Box<Expression>),
    CallableReference {
        left: Option<String>,
        right: String,
    },
    Prefix {
        annotation: Option<String>,
        label: Option<Label>,
        operator: Option<PrefixUnaryOperator>,
        expression: Box<Expression>,
    },
    Postfix {
        operator: PostfixUnaryOperator,
        expression: Box<Expression>,
    },
    Comparison {
        left: Box<Expression>,
        operator: ComparisonOperator,
        right: Box<Expression>,
    },
    Try {
        block: Vec<Statement>,
        catch_blocks: Vec<CatchBlock>,
        finally_block: Option<FinallyBlock>,
    },
    Parenthesized(Box<Expression>),
    Indexing(Box<Expression>, IndexingSuffix),
    This {
        identifier: Option<String>,
    },
    Super,
}

impl Expression {
    pub fn new(node: &Node, content: &[u8]) -> Result<Expression> {
        match node.kind() {
            "call_expression" => call_expression(node, content),
            "navigation_expression" => navigation_expression(node, content),
            "if_expression" => if_expression(node, content),
            "disjunction_expression" => disjunction_expression(node, content),
            "conjunction_expression" => conjunction_expression(node, content),
            "additive_expression" => additive_expression(node, content),
            "equality_expression" => equality_expression(node, content),
            "multiplicative_expression" => multiplicative_expression(node, content),
            "comparison_expression" => comparison_expression(node, content),
            "prefix_expression" => prefix_expression(node, content),
            "postfix_expression" => postfix_expression(node, content),
            "simple_identifier" => Ok(Expression::Identifier {
                identifier: node.utf8_text(content)?.to_string(),
            }),
            "try_expression" => r#try::expression(node, content),
            "infix_expression" => infix_expression(node, content),
            "as_expression" => as_expression(node, content),
            "elvis_expression" => elvis_expression(node, content),
            "range_expression" => range_expression(node, content),
            "check_expression" => check_expression(node, content),
            "super_expression" => Ok(Expression::Super),
            "callable_reference" => callable_reference(node, content),
            "boolean_literal" | "string_literal" | "integer_literal" | "object_literal"
            | "character_literal" | "lambda_literal" | "long_literal" | "real_literal"
            | "hex_literal" | "null" => Ok(Expression::Literal(Literal::new(node, content)?)),
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
            "parenthesized_expression" => Ok(Expression::Parenthesized(Box::new(Expression::new(
                &node.child(1).context(format!(
                    "[Expression::Parenthesized] no child at {}",
                    node.start_position()
                ))?,
                content,
            )?))),
            "indexing_expression" => indexing_expression(node, content),
            "this_expression" => this_expression(node, content),
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

fn prefix_expression(node: &Node, content: &[u8]) -> Result<Expression> {
    let child = node.child(0).context(format!(
        "[Expression::Prefix] no child at {}",
        node.start_position()
    ))?;

    let (annotation, label, operator) = match child.kind() {
        "annotation" => (Some(child.utf8_text(content)?.to_string()), None, None),
        "label" => (None, Some(Label::new(&child, content)?), None),
        "++" => (None, None, Some(PrefixUnaryOperator::Increment)),
        "--" => (None, None, Some(PrefixUnaryOperator::Decrement)),
        "-" => (None, None, Some(PrefixUnaryOperator::Minus)),
        "+" => (None, None, Some(PrefixUnaryOperator::Plus)),
        "!" => (None, None, Some(PrefixUnaryOperator::Negation)),
        _ => bail!(
            "[Expression::Prefix] unknonwn child at {}",
            child.start_position()
        ),
    };
    Ok(Expression::Prefix {
        annotation,
        label,
        operator,
        expression: Box::new(Expression::new(
            &node.child(1).context(format!(
                "[Expression::Prefix] no expression found at {}",
                node.start_position()
            ))?,
            content,
        )?),
    })
}

fn postfix_expression(node: &Node, content: &[u8]) -> Result<Expression> {
    let child = node.child(1).context(format!(
        "[Expression::Postfix] no child at {}",
        node.start_position()
    ))?;

    let operator = match child.kind() {
        "++" => PostfixUnaryOperator::Increment,
        "--" => PostfixUnaryOperator::Decrement,
        "!!" => PostfixUnaryOperator::NullAssertion,
        _ => bail!(
            "[Expression::Postfix] unknonwn child at {}",
            child.start_position()
        ),
    };
    Ok(Expression::Postfix {
        operator,
        expression: Box::new(Expression::new(
            &node.child(0).context(format!(
                "[Expression::Postfix] no expression found at {}",
                node.start_position()
            ))?,
            content,
        )?),
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

fn multiplicative_expression(node: &Node, content: &[u8]) -> Result<Expression> {
    Ok(Expression::Multiplicative {
        left: Box::new(Expression::new(
            &node.child(0).context(format!(
                "[Expression::Multiplicative] no expression found at {}",
                node.start_position()
            ))?,
            content,
        )?),
        operator: MultiplicativeOperator::new(
            &node.child(1).context(format!(
                "[Expression::Multiplicative] no operator found at {}",
                node.start_position()
            ))?,
            content,
        )?,
        right: Box::new(Expression::new(
            &node.child(2).context(format!(
                "[Expression::Multiplicative] no expression found at {}",
                node.start_position()
            ))?,
            content,
        )?),
    })
}

fn comparison_expression(node: &Node, content: &[u8]) -> Result<Expression> {
    Ok(Expression::Comparison {
        left: Box::new(Expression::new(
            &node.child(0).context(format!(
                "[Expression::Comparison] no expression found at {}",
                node.start_position()
            ))?,
            content,
        )?),
        operator: ComparisonOperator::new(
            &node.child(1).context(format!(
                "[Expression::Comparison] no operator found at {}",
                node.start_position()
            ))?,
            content,
        )?,
        right: Box::new(Expression::new(
            &node.child(2).context(format!(
                "[Expression::Comparison] no expression found at {}",
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

fn additive_expression(node: &Node, content: &[u8]) -> Result<Expression> {
    Ok(Expression::Conjunction {
        left: Box::new(Expression::new(
            &node.child(0).context(format!(
                "[Expression::Additive] no expression found at {}",
                node.start_position()
            ))?,
            content,
        )?),
        right: Box::new(Expression::new(
            &node.child(2).context(format!(
                "[Expression::Additive] no expression found at {}",
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

fn range_expression(node: &Node, content: &[u8]) -> Result<Expression> {
    Ok(Expression::Range {
        left: Box::new(Expression::new(
            &node.child(0).context(format!(
                "[Expression::Range] too little children at {}",
                node.start_position()
            ))?,
            content,
        )?),
        right: Box::new(Expression::new(
            &node.child(2).context(format!(
                "[Expression::Range] too little children at {}",
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
        "!in" => Expression::CheckNotIn {
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
        "is" => Expression::CheckIs {
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
        "!is" => Expression::CheckNotIs {
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
        Ok(WhenSubject {
            expression: Box::new(Expression::new(
                &node.child(node.child_count() - 2).context(format!(
                    "[WhenSubject] no child at {}",
                    node.start_position()
                ))?,
                content,
            )?),
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
            "range_test" => WhenCondition::RangeTest(Expression::new(
                &child.child(1).context(format!(
                    "[WhenCondition] no child 1 at {}",
                    node.start_position(),
                ))?,
                content,
            )?),
            "type_test" => WhenCondition::TypeTest(Type::new(
                &child.child(1).context(format!(
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
                "when_condition" => condition = Some(WhenCondition::new(&child, content)?),
                "control_structure_body" => {
                    body = Some(ControlStructureBody::new(&child, content)?)
                }
                _ => {}
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

fn indexing_expression(node: &Node, content: &[u8]) -> Result<Expression> {
    Ok(Expression::Indexing(
        Box::new(Expression::new(
            &node.child(0).context(format!(
                "[Expression::Indexing] no child at {}",
                node.start_position()
            ))?,
            content,
        )?),
        IndexingSuffix::new(
            &node.child(1).context(format!(
                "[Expression::Indexing] no child at {}",
                node.start_position()
            ))?,
            content,
        )?,
    ))
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct IndexingSuffix {
    pub expressions: Vec<Expression>,
}

impl IndexingSuffix {
    fn new(node: &Node, content: &[u8]) -> Result<IndexingSuffix> {
        let mut expressions = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "[" | "," | "]" => {}
                _ => expressions.push(Expression::new(&child, content)?),
            }
        }

        Ok(IndexingSuffix { expressions })
    }
}

fn this_expression(node: &Node, content: &[u8]) -> Result<Expression> {
    Ok(
        match node
            .child(0)
            .context(format!(
                "[Expression::This] no child at {}",
                node.start_position()
            ))?
            .kind()
        {
            "this" => Expression::This { identifier: None },
            "this@" => Expression::This {
                identifier: Some(
                    node.child(1)
                        .context(format!(
                            "[Expression::This] no child at {}",
                            node.start_position()
                        ))?
                        .utf8_text(content)?
                        .to_string(),
                ),
            },
            this => {
                bail!(
                    "[Expression::This] unhandled this {} at {}",
                    this,
                    node.start_position(),
                )
            }
        },
    )
}
