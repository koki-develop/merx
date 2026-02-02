mod error;

use std::collections::HashMap;

use pest::Parser;
use pest::iterators::Pair;
use pest_derive::Parser;

pub use error::ParseError;

use crate::ast::{
    BinaryOp, Direction, Edge, EdgeLabel, Expr, Flowchart, Node, Statement, TypeName, UnaryOp,
};

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct MermaidParser;

pub fn parse(input: &str) -> Result<Flowchart, ParseError> {
    let pairs = MermaidParser::parse(Rule::flowchart, input)?;

    let mut direction = Direction::Td;
    let mut nodes: HashMap<String, Node> = HashMap::new();
    let mut edges: Vec<Edge> = Vec::new();

    for pair in pairs {
        if pair.as_rule() == Rule::flowchart {
            for inner in pair.into_inner() {
                match inner.as_rule() {
                    Rule::direction => {
                        direction = parse_direction(inner);
                    }
                    Rule::line => {
                        let (from_id, to_id, label, from_node, to_node) = parse_line(inner)?;

                        if let Some(node) = from_node {
                            nodes.entry(node.id().to_string()).or_insert(node);
                        }
                        if let Some(node) = to_node {
                            nodes.entry(node.id().to_string()).or_insert(node);
                        }

                        edges.push(Edge {
                            from: from_id,
                            to: to_id,
                            label,
                        });
                    }
                    _ => {}
                }
            }
        }
    }

    // Validate: condition nodes must have both Yes and No edges
    for node in nodes.values() {
        if let Node::Condition { id, .. } = node {
            let mut has_yes = false;
            let mut has_no = false;

            for edge in &edges {
                if &edge.from != id {
                    continue;
                }

                match &edge.label {
                    Some(EdgeLabel::Yes) => {
                        if has_yes {
                            return Err(ParseError::new(format!(
                                "Condition node '{}' has multiple 'Yes' edges",
                                id
                            )));
                        }
                        has_yes = true;
                    }
                    Some(EdgeLabel::No) => {
                        if has_no {
                            return Err(ParseError::new(format!(
                                "Condition node '{}' has multiple 'No' edges",
                                id
                            )));
                        }
                        has_no = true;
                    }
                    Some(EdgeLabel::Custom(s)) => {
                        return Err(ParseError::new(format!(
                            "Condition node '{}' must have 'Yes' or 'No' label, but got '{}'",
                            id, s
                        )));
                    }
                    None => {
                        return Err(ParseError::new(format!(
                            "Edge from condition node '{}' must have 'Yes' or 'No' label",
                            id
                        )));
                    }
                }
            }

            if !has_yes {
                return Err(ParseError::new(format!(
                    "Condition node '{}' is missing 'Yes' edge",
                    id
                )));
            }
            if !has_no {
                return Err(ParseError::new(format!(
                    "Condition node '{}' is missing 'No' edge",
                    id
                )));
            }
        }
    }

    let nodes_vec: Vec<Node> = nodes.into_values().collect();

    Ok(Flowchart {
        direction,
        nodes: nodes_vec,
        edges,
    })
}

fn parse_direction(pair: Pair<Rule>) -> Direction {
    match pair.as_str() {
        "TD" => Direction::Td,
        "TB" => Direction::Tb,
        "LR" => Direction::Lr,
        "RL" => Direction::Rl,
        "BT" => Direction::Bt,
        _ => Direction::Td,
    }
}

fn parse_line(
    pair: Pair<Rule>,
) -> Result<
    (
        String,
        String,
        Option<EdgeLabel>,
        Option<Node>,
        Option<Node>,
    ),
    ParseError,
> {
    let edge_def = pair.into_inner().next().unwrap();
    let mut inner = edge_def.into_inner();

    let from_pair = inner.next().unwrap();
    let (from_id, from_node) = parse_node_ref(from_pair)?;

    let mut label = None;
    let mut to_pair = inner.next().unwrap();

    if to_pair.as_rule() == Rule::edge_label {
        label = Some(parse_edge_label(to_pair));
        to_pair = inner.next().unwrap();
    }

    let (to_id, to_node) = parse_node_ref(to_pair)?;

    Ok((from_id, to_id, label, from_node, to_node))
}

fn parse_edge_label(pair: Pair<Rule>) -> EdgeLabel {
    let label_text = pair.into_inner().next().unwrap().as_str();
    match label_text.to_lowercase().as_str() {
        "yes" => EdgeLabel::Yes,
        "no" => EdgeLabel::No,
        _ => EdgeLabel::Custom(label_text.to_string()),
    }
}

fn parse_node_ref(pair: Pair<Rule>) -> Result<(String, Option<Node>), ParseError> {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::node_with_def => {
            let node = parse_node_with_def(inner)?;
            let id = node.id().to_string();
            Ok((id, Some(node)))
        }
        Rule::bare_identifier => {
            let id = inner.as_str().to_string();
            Ok((id, None))
        }
        _ => unreachable!(),
    }
}

fn parse_node_with_def(pair: Pair<Rule>) -> Result<Node, ParseError> {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::start_node => Ok(Node::Start),
        Rule::end_node => Ok(Node::End),
        Rule::process_node => {
            let mut parts = inner.into_inner();
            let id = parts.next().unwrap().as_str().to_string();
            let statements_pair = parts.next().unwrap();
            let statements = parse_statements(statements_pair)?;
            Ok(Node::Process { id, statements })
        }
        Rule::condition_node => {
            let mut parts = inner.into_inner();
            let id = parts.next().unwrap().as_str().to_string();
            let expr_pair = parts.next().unwrap();
            let condition = parse_expression(expr_pair)?;
            Ok(Node::Condition { id, condition })
        }
        _ => unreachable!(),
    }
}

fn parse_statements(pair: Pair<Rule>) -> Result<Vec<Statement>, ParseError> {
    let mut statements = Vec::new();

    for stmt_pair in pair.into_inner() {
        if stmt_pair.as_rule() == Rule::statement {
            statements.push(parse_statement(stmt_pair)?);
        }
    }

    Ok(statements)
}

fn parse_statement(pair: Pair<Rule>) -> Result<Statement, ParseError> {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::print_stmt => {
            let expr_pair = inner.into_inner().next().unwrap();
            let expr = parse_expression(expr_pair)?;
            Ok(Statement::Print { expr })
        }
        Rule::error_stmt => {
            let expr_pair = inner.into_inner().next().unwrap();
            let message = parse_expression(expr_pair)?;
            Ok(Statement::Error { message })
        }
        Rule::assign_stmt => {
            let mut parts = inner.into_inner();
            let variable = parts.next().unwrap().as_str().to_string();
            let value = parse_expression(parts.next().unwrap())?;
            Ok(Statement::Assign { variable, value })
        }
        _ => unreachable!(),
    }
}

fn parse_expression(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    let mut operands: Vec<Expr> = Vec::new();
    let mut operators: Vec<BinaryOp> = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::unary_expr => {
                operands.push(parse_unary_expr(inner)?);
            }
            Rule::binary_op => {
                operators.push(parse_binary_op(inner));
            }
            _ => {}
        }
    }

    if operands.is_empty() {
        unreachable!("Expression must have at least one operand");
    }

    // Build AST with proper precedence using shunting-yard
    build_expr_with_precedence(operands, operators)
}

fn precedence(op: &BinaryOp) -> u8 {
    match op {
        BinaryOp::Or => 1,
        BinaryOp::And => 2,
        BinaryOp::Eq | BinaryOp::Ne => 3,
        BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => 4,
        BinaryOp::Add | BinaryOp::Sub => 5,
        BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => 6,
    }
}

fn build_expr_with_precedence(
    mut operands: Vec<Expr>,
    operators: Vec<BinaryOp>,
) -> Result<Expr, ParseError> {
    if operators.is_empty() {
        return Ok(operands.remove(0));
    }

    // Find the lowest precedence operator (rightmost for left associativity)
    let mut min_prec = u8::MAX;
    let mut min_idx = 0;

    for (i, op) in operators.iter().enumerate() {
        let prec = precedence(op);
        if prec <= min_prec {
            min_prec = prec;
            min_idx = i;
        }
    }

    let op = operators[min_idx];

    let left_operands: Vec<Expr> = operands.drain(..=min_idx).collect();
    let left_operators: Vec<BinaryOp> = operators[..min_idx].to_vec();
    let right_operands: Vec<Expr> = operands;
    let right_operators: Vec<BinaryOp> = operators[min_idx + 1..].to_vec();

    let left = build_expr_with_precedence(left_operands, left_operators)?;
    let right = build_expr_with_precedence(right_operands, right_operators)?;

    Ok(Expr::Binary {
        op,
        left: Box::new(left),
        right: Box::new(right),
    })
}

fn parse_unary_expr(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    let mut unary_ops: Vec<UnaryOp> = Vec::new();
    let mut cast_expr = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::unary_op => {
                unary_ops.push(match inner.as_str() {
                    "!" => UnaryOp::Not,
                    "-" => UnaryOp::Neg,
                    _ => unreachable!(),
                });
            }
            Rule::cast_expr => {
                cast_expr = Some(parse_cast_expr(inner)?);
            }
            _ => {}
        }
    }

    let mut expr = cast_expr.unwrap();

    // Apply unary operators in reverse order
    for op in unary_ops.into_iter().rev() {
        expr = Expr::Unary {
            op,
            operand: Box::new(expr),
        };
    }

    Ok(expr)
}

fn parse_cast_expr(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    let mut expr = None;
    let mut target_type = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::primary => {
                expr = Some(parse_primary(inner)?);
            }
            Rule::as_keyword => {}
            Rule::type_name => {
                target_type = Some(match inner.as_str() {
                    "int" => TypeName::Int,
                    "str" => TypeName::Str,
                    _ => unreachable!(),
                });
            }
            _ => {}
        }
    }

    let mut result = expr.unwrap();

    if let Some(t) = target_type {
        result = Expr::Cast {
            expr: Box::new(result),
            target_type: t,
        };
    }

    Ok(result)
}

fn parse_primary(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::expression => parse_expression(inner),
        Rule::input_keyword => Ok(Expr::Input),
        Rule::bool_lit => Ok(Expr::BoolLit {
            value: inner.as_str() == "true",
        }),
        Rule::int_lit => Ok(Expr::IntLit {
            value: inner.as_str().parse().unwrap(),
        }),
        Rule::string_lit => {
            let s = inner.as_str();
            // Remove surrounding quotes
            let content = &s[1..s.len() - 1];
            Ok(Expr::StrLit {
                value: content.to_string(),
            })
        }
        Rule::identifier => Ok(Expr::Variable {
            name: inner.as_str().to_string(),
        }),
        _ => unreachable!(),
    }
}

fn parse_binary_op(pair: Pair<Rule>) -> BinaryOp {
    match pair.as_str() {
        "+" => BinaryOp::Add,
        "-" => BinaryOp::Sub,
        "*" => BinaryOp::Mul,
        "/" => BinaryOp::Div,
        "%" => BinaryOp::Mod,
        "==" => BinaryOp::Eq,
        "!=" => BinaryOp::Ne,
        "<" => BinaryOp::Lt,
        "<=" => BinaryOp::Le,
        ">" => BinaryOp::Gt,
        ">=" => BinaryOp::Ge,
        "&&" => BinaryOp::And,
        "||" => BinaryOp::Or,
        _ => unreachable!(),
    }
}
