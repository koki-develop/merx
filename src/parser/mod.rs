//! Parser module for Mermaid flowchart syntax.
//!
//! This module provides parsing functionality to convert Mermaid flowchart
//! text into an Abstract Syntax Tree (AST) representation. It uses the
//! [pest](https://pest.rs) parsing library with a PEG grammar defined in
//! `src/grammar.pest`.
//!
//! # Overview
//!
//! The parser transforms text in Mermaid flowchart format into a [`Flowchart`]
//! AST structure. The parsing process includes:
//!
//! 1. Lexical and syntactic analysis using pest
//! 2. AST construction with proper operator precedence handling
//! 3. Semantic validation (e.g., condition node edge requirements)
//!
//! # Usage
//!
//! ```
//! use merx::parser::parse;
//!
//! let input = r#"flowchart TD
//!     Start --> A[x = 1]
//!     A --> End
//! "#;
//!
//! let flowchart = parse(input).expect("Failed to parse");
//! ```
//!
//! # Grammar
//!
//! The parser supports the following Mermaid flowchart constructs:
//!
//! - **Directions**: `TD`, `TB`, `LR`, `RL`, `BT`
//! - **Nodes**: `Start`, `End`, process nodes `id[statements]`, condition nodes `id{expr?}`
//! - **Edges**: `-->` with optional labels `|Yes|`, `|No|`, or custom text
//! - **Expressions**: Arithmetic, comparison, logical operators with proper precedence
//! - **Statements**: `println`, `error`, and assignment (`=`)
//!
//! # See Also
//!
//! - [`Flowchart`] - The root AST type produced by parsing
//! - [`ParseError`] - Error type returned on parse failures
//! - [pest documentation](https://pest.rs/book/)

mod error;

use std::collections::HashMap;

use pest::Parser;
use pest::iterators::Pair;
use pest_derive::Parser;

pub use error::ParseError;

use crate::ast::{
    BinaryOp, Direction, Edge, EdgeLabel, Expr, Flowchart, Node, Statement, TypeName, UnaryOp,
};

/// Internal pest parser generated from the PEG grammar.
///
/// This struct is derived by `pest_derive` and implements the [`pest::Parser`]
/// trait. The grammar rules are defined in `src/grammar.pest` and compiled
/// at build time into a `Rule` enum.
#[derive(Parser)]
#[grammar = "grammar.pest"]
struct MermaidParser;

/// Parses Mermaid flowchart source text into an AST.
///
/// This is the main entry point for parsing Mermaid flowchart programs.
/// It performs complete parsing including lexical analysis, syntax tree
/// construction, and semantic validation.
///
/// # Arguments
///
/// * `input` - The Mermaid flowchart source code as a string slice
///
/// # Returns
///
/// Returns `Ok(Flowchart)` on successful parsing, containing the complete
/// AST representation of the flowchart.
///
/// # Errors
///
/// Returns [`ParseError`] in the following cases:
///
/// - **Syntax errors**: Invalid Mermaid flowchart syntax
/// - **Missing edges**: Condition nodes without both `Yes` and `No` edges
/// - **Duplicate edges**: Condition nodes with multiple `Yes` or `No` edges
/// - **Invalid labels**: Condition node edges with custom labels instead of `Yes`/`No`
///
/// # Examples
///
/// ```
/// use merx::parser::parse;
///
/// // Simple flowchart with assignment
/// let input = r#"flowchart TD
///     Start --> A[x = 42]
///     A --> End
/// "#;
/// let flowchart = parse(input).unwrap();
/// assert_eq!(flowchart.nodes.len(), 3);
///
/// // Flowchart with condition
/// let input = r#"flowchart TD
///     Start --> A{x > 0?}
///     A -->|Yes| B[println x]
///     A -->|No| End
///     B --> End
/// "#;
/// let flowchart = parse(input).unwrap();
/// ```
///
/// # Validation
///
/// After parsing, the function validates that all condition nodes have
/// exactly one `Yes` edge and one `No` edge. This ensures the flowchart
/// can be executed without ambiguity.
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
                        let parsed = parse_line(inner)?;

                        if let Some(node) = parsed.from_node {
                            nodes.entry(node.id().to_string()).or_insert(node);
                        }
                        if let Some(node) = parsed.to_node {
                            nodes.entry(node.id().to_string()).or_insert(node);
                        }

                        edges.push(Edge {
                            from: parsed.from_id,
                            to: parsed.to_id,
                            label: parsed.label,
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

    // Validate: End node must not have outgoing edges
    for edge in &edges {
        if edge.from == "End" {
            return Err(ParseError::new("End node cannot have outgoing edges"));
        }
    }

    // Validate: Non-condition nodes must have at most one outgoing edge
    let mut edge_counts: HashMap<String, usize> = HashMap::new();
    for edge in &edges {
        *edge_counts.entry(edge.from.clone()).or_insert(0) += 1;
    }
    for (node_id, count) in edge_counts {
        if count > 1 {
            // Condition nodes are allowed to have 2 edges (Yes and No)
            let is_condition = nodes
                .get(&node_id)
                .is_some_and(|n| matches!(n, Node::Condition { .. }));
            if !is_condition {
                return Err(ParseError::new(format!(
                    "Node '{}' has multiple outgoing edges (expected at most 1)",
                    node_id
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

/// Converts a direction token into a [`Direction`] enum value.
///
/// # Arguments
///
/// * `pair` - A pest [`Pair`] matching the `direction` rule
///
/// # Returns
///
/// The corresponding [`Direction`] variant. Defaults to [`Direction::Td`]
/// for unrecognized values (should not occur with valid grammar).
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

/// Result of parsing a single line (edge definition) in the flowchart.
///
/// Contains the edge information and any node definitions found on the line.
struct ParsedLine {
    /// The source node identifier.
    from_id: String,
    /// The target node identifier.
    to_id: String,
    /// Optional edge label (`Yes`, `No`, or custom text).
    label: Option<EdgeLabel>,
    /// The source node definition, if present on this line.
    from_node: Option<Node>,
    /// The target node definition, if present on this line.
    to_node: Option<Node>,
}

/// Parses a single line containing an edge definition.
///
/// A line in the flowchart defines an edge between two nodes, optionally
/// with a label. Either or both nodes may include their full definition
/// (shape and content) or just reference an existing node by ID.
///
/// # Arguments
///
/// * `pair` - A pest [`Pair`] matching the `line` rule
///
/// # Returns
///
/// A [`ParsedLine`] containing:
/// - `from_id`: The source node identifier
/// - `to_id`: The target node identifier
/// - `label`: Optional edge label (`Yes`, `No`, or custom)
/// - `from_node`: The source node definition if present in this line
/// - `to_node`: The target node definition if present in this line
///
/// # Errors
///
/// Returns [`ParseError`] if node parsing fails.
fn parse_line(pair: Pair<Rule>) -> Result<ParsedLine, ParseError> {
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

    Ok(ParsedLine {
        from_id,
        to_id,
        label,
        from_node,
        to_node,
    })
}

/// Parses an edge label enclosed in `|` delimiters.
///
/// Labels are case-insensitive for `Yes` and `No`. Any other label
/// text is preserved as a custom label.
///
/// # Arguments
///
/// * `pair` - A pest [`Pair`] matching the `edge_label` rule
///
/// # Returns
///
/// An [`EdgeLabel`] variant:
/// - [`EdgeLabel::Yes`] for "yes" (case-insensitive)
/// - [`EdgeLabel::No`] for "no" (case-insensitive)
/// - [`EdgeLabel::Custom`] for any other text
fn parse_edge_label(pair: Pair<Rule>) -> EdgeLabel {
    let label_text = pair.into_inner().next().unwrap().as_str();
    match label_text.to_lowercase().as_str() {
        "yes" => EdgeLabel::Yes,
        "no" => EdgeLabel::No,
        _ => EdgeLabel::Custom(label_text.to_string()),
    }
}

/// Parses a node reference, which may be a full definition or a bare identifier.
///
/// In Mermaid flowcharts, nodes can be defined inline with their content
/// (e.g., `A[println x]`) or referenced by just their ID (e.g., `A`).
///
/// # Arguments
///
/// * `pair` - A pest [`Pair`] matching the `node_ref` rule
///
/// # Returns
///
/// A tuple containing:
/// - The node identifier as a `String`
/// - `Some(Node)` if this reference includes a node definition, `None` otherwise
///
/// # Errors
///
/// Returns [`ParseError`] if the node definition cannot be parsed.
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

/// Parses a node with its full definition (shape and content).
///
/// Handles the four node types supported by the grammar:
/// - `Start`: The entry point of the flowchart
/// - `End`: A termination point of the flowchart
/// - Process nodes: `id[statements]` - rectangular nodes with executable statements
/// - Condition nodes: `id{expr?}` - diamond nodes with a boolean expression
///
/// # Arguments
///
/// * `pair` - A pest [`Pair`] matching the `node_with_def` rule
///
/// # Returns
///
/// The parsed [`Node`] variant.
///
/// # Errors
///
/// Returns [`ParseError`] if statement or expression parsing fails.
fn parse_node_with_def(pair: Pair<Rule>) -> Result<Node, ParseError> {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::start_node => {
            let label = parse_stadium_label(&inner);
            Ok(Node::Start { label })
        }
        Rule::end_node => {
            let label = parse_stadium_label(&inner);
            Ok(Node::End { label })
        }
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

/// Parses the optional stadium label from a start or end node.
///
/// Stadium labels use the Mermaid syntax `([label text])` and provide
/// a display name for Start and End nodes.
///
/// # Arguments
///
/// * `pair` - A pest [`Pair`] matching `start_node` or `end_node` rule
///
/// # Returns
///
/// `Some(String)` containing the label text if present, `None` otherwise.
fn parse_stadium_label(pair: &Pair<Rule>) -> Option<String> {
    for inner in pair.clone().into_inner() {
        if inner.as_rule() == Rule::stadium_label {
            return inner.into_inner().next().map(|p| p.as_str().to_string());
        }
    }
    None
}

/// Parses a semicolon-separated list of statements.
///
/// Statements appear inside process nodes and are executed sequentially
/// when the node is visited during flowchart execution.
///
/// # Arguments
///
/// * `pair` - A pest [`Pair`] matching the `statements` rule
///
/// # Returns
///
/// A vector of parsed [`Statement`] values in order of appearance.
///
/// # Errors
///
/// Returns [`ParseError`] if any individual statement cannot be parsed.
fn parse_statements(pair: Pair<Rule>) -> Result<Vec<Statement>, ParseError> {
    let mut statements = Vec::new();

    for stmt_pair in pair.into_inner() {
        if stmt_pair.as_rule() == Rule::statement {
            statements.push(parse_statement(stmt_pair)?);
        }
    }

    Ok(statements)
}

/// Parses a single statement.
///
/// Supports four statement types:
/// - `println expr`: Outputs the expression value to stdout with newline
/// - `print expr`: Outputs the expression value to stdout without newline
/// - `error expr`: Outputs the expression value to stderr
/// - `variable = expr`: Assigns the expression value to a variable
///
/// # Arguments
///
/// * `pair` - A pest [`Pair`] matching the `statement` rule
///
/// # Returns
///
/// The parsed [`Statement`] variant.
///
/// # Errors
///
/// Returns [`ParseError`] if the expression within the statement cannot be parsed.
fn parse_statement(pair: Pair<Rule>) -> Result<Statement, ParseError> {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::println_stmt => {
            let expr_pair = inner.into_inner().next().unwrap();
            let expr = parse_expression(expr_pair)?;
            Ok(Statement::Println { expr })
        }
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

/// Parses an expression into an AST with correct operator precedence.
///
/// The grammar produces a flat sequence of operands and operators. This
/// function collects them and delegates to [`build_expr_with_precedence`]
/// to construct a properly nested AST respecting operator precedence.
///
/// # Arguments
///
/// * `pair` - A pest [`Pair`] matching the `expression` rule
///
/// # Returns
///
/// The parsed expression as an [`Expr`] tree.
///
/// # Errors
///
/// Returns [`ParseError`] if any sub-expression cannot be parsed.
///
/// # See Also
///
/// - [`build_expr_with_precedence`] - Handles precedence-based tree construction
/// - [`precedence`] - Defines operator precedence levels
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

    // Build AST with proper operator precedence
    build_expr_with_precedence(operands, operators)
}

/// Returns the precedence level for a binary operator.
///
/// Higher values indicate higher precedence (tighter binding).
/// Operators at the same level have equal precedence.
///
/// # Precedence Levels
///
/// | Level | Operators | Description |
/// |-------|-----------|-------------|
/// | 6 | `*`, `/`, `%` | Multiplicative |
/// | 5 | `+`, `-` | Additive |
/// | 4 | `<`, `<=`, `>`, `>=` | Relational |
/// | 3 | `==`, `!=` | Equality |
/// | 2 | `&&` | Logical AND |
/// | 1 | `\|\|` | Logical OR |
///
/// # Arguments
///
/// * `op` - The binary operator to get precedence for
///
/// # Returns
///
/// A `u8` precedence value where higher means tighter binding.
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

/// Builds an expression AST with correct operator precedence.
///
/// Uses a recursive descent approach to construct a properly nested binary
/// expression tree. The algorithm finds the lowest-precedence operator
/// (rightmost when tied, for left-associativity), splits the expression
/// at that point, and recursively processes each side.
///
/// # Algorithm
///
/// 1. If no operators remain, return the single operand
/// 2. Find the rightmost operator with the lowest precedence
/// 3. Split operands and operators at that position
/// 4. Recursively build left and right subtrees
/// 5. Combine into a [`Expr::Binary`] node
///
/// # Arguments
///
/// * `operands` - Vector of parsed sub-expressions (unary expressions)
/// * `operators` - Vector of binary operators between operands
///
/// # Returns
///
/// A single [`Expr`] representing the properly nested expression tree.
///
/// # Errors
///
/// Returns [`ParseError`] if recursive parsing fails (propagated from callers).
///
/// # Example
///
/// For input `1 + 2 * 3`:
/// - operands: `[1, 2, 3]`
/// - operators: `[+, *]`
/// - Result: `Binary(+, 1, Binary(*, 2, 3))`
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

/// Parses a unary expression (prefix operators applied to a cast expression).
///
/// Handles zero or more prefix unary operators (`!` for logical NOT, `-` for
/// negation) followed by a cast expression. Operators are applied innermost
/// first (right-to-left), so `--x` becomes `Unary(Neg, Unary(Neg, x))`.
///
/// # Arguments
///
/// * `pair` - A pest [`Pair`] matching the `unary_expr` rule
///
/// # Returns
///
/// The parsed expression with unary operators applied.
///
/// # Errors
///
/// Returns [`ParseError`] if the inner cast expression cannot be parsed.
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

/// Parses a cast expression (primary with optional type cast).
///
/// Handles expressions like `x as int` or `input as str`. If no `as` keyword
/// is present, returns the primary expression unchanged.
///
/// # Supported Types
///
/// - `int`: Integer type
/// - `str`: String type
///
/// Note: Casting to `bool` is not supported in the language syntax.
/// Also, `bool as int` will result in a runtime error.
///
/// # Arguments
///
/// * `pair` - A pest [`Pair`] matching the `cast_expr` rule
///
/// # Returns
///
/// The parsed expression, wrapped in [`Expr::Cast`] if a type cast is present.
///
/// # Errors
///
/// Returns [`ParseError`] if the primary expression cannot be parsed.
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

/// Parses a primary expression (atoms and parenthesized expressions).
///
/// Primary expressions are the building blocks of larger expressions:
/// - Parenthesized expressions: `(expr)`
/// - Input keyword: `input` (reads from stdin at runtime)
/// - Boolean literals: `true`, `false`
/// - Integer literals: sequences of digits
/// - String literals: single-quoted strings (e.g., `'hello'`)
/// - Variables: identifiers referring to stored values
///
/// # Arguments
///
/// * `pair` - A pest [`Pair`] matching the `primary` rule
///
/// # Returns
///
/// The parsed [`Expr`] variant corresponding to the primary type.
///
/// # Errors
///
/// Returns [`ParseError`] if a nested expression cannot be parsed.
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

/// Converts a binary operator token into a [`BinaryOp`] enum value.
///
/// # Supported Operators
///
/// | Token | Variant | Description |
/// |-------|---------|-------------|
/// | `+` | `Add` | Addition |
/// | `-` | `Sub` | Subtraction |
/// | `*` | `Mul` | Multiplication |
/// | `/` | `Div` | Division |
/// | `%` | `Mod` | Modulo |
/// | `==` | `Eq` | Equality |
/// | `!=` | `Ne` | Inequality |
/// | `<` | `Lt` | Less than |
/// | `<=` | `Le` | Less than or equal |
/// | `>` | `Gt` | Greater than |
/// | `>=` | `Ge` | Greater than or equal |
/// | `&&` | `And` | Logical AND |
/// | `\|\|` | `Or` | Logical OR |
///
/// # Arguments
///
/// * `pair` - A pest [`Pair`] matching the `binary_op` rule
///
/// # Returns
///
/// The corresponding [`BinaryOp`] variant.
///
/// # Panics
///
/// Panics with `unreachable!()` if an unknown operator is encountered
/// (should not happen with a valid grammar).
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

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to parse an expression from a condition node
    fn parse_condition_expr(expr_str: &str) -> Expr {
        let input = format!(
            r#"flowchart TD
    Start --> A{{{}?}}
    A -->|Yes| End
    A -->|No| End
"#,
            expr_str
        );
        let flowchart = parse(&input).unwrap();
        let condition_node = flowchart
            .nodes
            .iter()
            .find(|n| matches!(n, Node::Condition { .. }))
            .unwrap();
        match condition_node {
            Node::Condition { condition, .. } => condition.clone(),
            _ => unreachable!(),
        }
    }

    // Helper function to parse an expression from an assignment statement
    fn parse_assign_expr(expr_str: &str) -> Expr {
        let input = format!(
            r#"flowchart TD
    Start --> A[result = {}]
    A --> End
"#,
            expr_str
        );
        let flowchart = parse(&input).unwrap();
        let process_node = flowchart
            .nodes
            .iter()
            .find(|n| matches!(n, Node::Process { .. }))
            .unwrap();
        match process_node {
            Node::Process { statements, .. } => match &statements[0] {
                Statement::Assign { value, .. } => value.clone(),
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_parse_nested_parentheses() {
        // (1 + 2) * (3 - 4)
        let expr = parse_assign_expr("(1 + 2) * (3 - 4)");

        // Should be: Mul((1 + 2), (3 - 4))
        match expr {
            Expr::Binary { op, left, right } => {
                assert!(matches!(op, BinaryOp::Mul));

                // left: (1 + 2)
                match *left {
                    Expr::Binary { op, left, right } => {
                        assert!(matches!(op, BinaryOp::Add));
                        assert!(matches!(*left, Expr::IntLit { value: 1 }));
                        assert!(matches!(*right, Expr::IntLit { value: 2 }));
                    }
                    _ => panic!("Expected Binary for left operand"),
                }

                // right: (3 - 4)
                match *right {
                    Expr::Binary { op, left, right } => {
                        assert!(matches!(op, BinaryOp::Sub));
                        assert!(matches!(*left, Expr::IntLit { value: 3 }));
                        assert!(matches!(*right, Expr::IntLit { value: 4 }));
                    }
                    _ => panic!("Expected Binary for right operand"),
                }
            }
            _ => panic!("Expected Binary expression"),
        }
    }

    #[test]
    fn test_parse_multiple_unary_operators_neg() {
        // --x (double negation)
        let expr = parse_assign_expr("--x");

        // Should be: Neg(Neg(x))
        match expr {
            Expr::Unary { op, operand } => {
                assert!(matches!(op, UnaryOp::Neg));
                match *operand {
                    Expr::Unary { op, operand } => {
                        assert!(matches!(op, UnaryOp::Neg));
                        match *operand {
                            Expr::Variable { name } => assert_eq!(name, "x"),
                            _ => panic!("Expected Variable"),
                        }
                    }
                    _ => panic!("Expected Unary for inner operand"),
                }
            }
            _ => panic!("Expected Unary expression"),
        }
    }

    #[test]
    fn test_parse_multiple_unary_operators_not() {
        // !!b (double logical NOT)
        let expr = parse_condition_expr("!!b");

        // Should be: Not(Not(b))
        match expr {
            Expr::Unary { op, operand } => {
                assert!(matches!(op, UnaryOp::Not));
                match *operand {
                    Expr::Unary { op, operand } => {
                        assert!(matches!(op, UnaryOp::Not));
                        match *operand {
                            Expr::Variable { name } => assert_eq!(name, "b"),
                            _ => panic!("Expected Variable"),
                        }
                    }
                    _ => panic!("Expected Unary for inner operand"),
                }
            }
            _ => panic!("Expected Unary expression"),
        }
    }

    #[test]
    fn test_parse_mixed_operators() {
        // 1 + 2 * 3 - 4
        // Should parse as: (1 + (2 * 3)) - 4 due to precedence
        // Tree: Sub(Add(1, Mul(2, 3)), 4)
        let expr = parse_assign_expr("1 + 2 * 3 - 4");

        match expr {
            Expr::Binary { op, left, right } => {
                assert!(matches!(op, BinaryOp::Sub));

                // right: 4
                assert!(matches!(*right, Expr::IntLit { value: 4 }));

                // left: 1 + (2 * 3)
                match *left {
                    Expr::Binary { op, left, right } => {
                        assert!(matches!(op, BinaryOp::Add));
                        assert!(matches!(*left, Expr::IntLit { value: 1 }));

                        // right of Add: 2 * 3
                        match *right {
                            Expr::Binary { op, left, right } => {
                                assert!(matches!(op, BinaryOp::Mul));
                                assert!(matches!(*left, Expr::IntLit { value: 2 }));
                                assert!(matches!(*right, Expr::IntLit { value: 3 }));
                            }
                            _ => panic!("Expected Mul"),
                        }
                    }
                    _ => panic!("Expected Add"),
                }
            }
            _ => panic!("Expected Binary expression"),
        }
    }

    #[test]
    fn test_parse_comparison_chain() {
        // x > 1 && x < 10
        // Should parse as: (x > 1) && (x < 10)
        let expr = parse_condition_expr("x > 1 && x < 10");

        match expr {
            Expr::Binary { op, left, right } => {
                assert!(matches!(op, BinaryOp::And));

                // left: x > 1
                match *left {
                    Expr::Binary { op, left, right } => {
                        assert!(matches!(op, BinaryOp::Gt));
                        match *left {
                            Expr::Variable { name } => assert_eq!(name, "x"),
                            _ => panic!("Expected Variable x"),
                        }
                        assert!(matches!(*right, Expr::IntLit { value: 1 }));
                    }
                    _ => panic!("Expected Gt"),
                }

                // right: x < 10
                match *right {
                    Expr::Binary { op, left, right } => {
                        assert!(matches!(op, BinaryOp::Lt));
                        match *left {
                            Expr::Variable { name } => assert_eq!(name, "x"),
                            _ => panic!("Expected Variable x"),
                        }
                        assert!(matches!(*right, Expr::IntLit { value: 10 }));
                    }
                    _ => panic!("Expected Lt"),
                }
            }
            _ => panic!("Expected Binary And expression"),
        }
    }

    #[test]
    fn test_parse_cast_in_expression() {
        // (x as int) + 1
        let expr = parse_assign_expr("(x as int) + 1");

        match expr {
            Expr::Binary { op, left, right } => {
                assert!(matches!(op, BinaryOp::Add));

                // left: x as int
                match *left {
                    Expr::Cast { expr, target_type } => {
                        assert!(matches!(target_type, TypeName::Int));
                        match *expr {
                            Expr::Variable { name } => assert_eq!(name, "x"),
                            _ => panic!("Expected Variable x"),
                        }
                    }
                    _ => panic!("Expected Cast"),
                }

                // right: 1
                assert!(matches!(*right, Expr::IntLit { value: 1 }));
            }
            _ => panic!("Expected Binary Add expression"),
        }
    }

    #[test]
    fn test_end_node_cannot_have_outgoing_edges() {
        let input = r#"flowchart TD
    Start --> End
    End --> A[x = 1]
    A --> End
"#;
        let result = parse(input);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.to_string(), "End node cannot have outgoing edges");
    }

    #[test]
    fn test_valid_flowchart_ending_at_end() {
        let input = r#"flowchart TD
    Start --> A[x = 1]
    A --> End
"#;
        let result = parse(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_process_node_multiple_edges_error() {
        let input = r#"flowchart TD
    Start --> A[x = 1]
    A --> B[y = 2]
    A --> C[z = 3]
    B --> End
    C --> End
"#;
        let result = parse(input);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(
            err.to_string(),
            "Node 'A' has multiple outgoing edges (expected at most 1)"
        );
    }

    #[test]
    fn test_start_node_multiple_edges_error() {
        let input = r#"flowchart TD
    Start --> A[x = 1]
    Start --> B[y = 2]
    A --> End
    B --> End
"#;
        let result = parse(input);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(
            err.to_string(),
            "Node 'Start' has multiple outgoing edges (expected at most 1)"
        );
    }

    #[test]
    fn test_condition_node_two_edges_allowed() {
        let input = r#"flowchart TD
    Start --> A{x > 0?}
    A -->|Yes| B[println x]
    A -->|No| End
    B --> End
"#;
        let result = parse(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_direction_td() {
        let input = r#"flowchart TD
    Start --> End
"#;
        let result = parse(input).unwrap();
        assert!(matches!(result.direction, Direction::Td));
    }

    #[test]
    fn test_parse_direction_tb() {
        let input = r#"flowchart TB
    Start --> End
"#;
        let result = parse(input).unwrap();
        assert!(matches!(result.direction, Direction::Tb));
    }

    #[test]
    fn test_parse_direction_lr() {
        let input = r#"flowchart LR
    Start --> End
"#;
        let result = parse(input).unwrap();
        assert!(matches!(result.direction, Direction::Lr));
    }

    #[test]
    fn test_parse_direction_rl() {
        let input = r#"flowchart RL
    Start --> End
"#;
        let result = parse(input).unwrap();
        assert!(matches!(result.direction, Direction::Rl));
    }

    #[test]
    fn test_parse_direction_bt() {
        let input = r#"flowchart BT
    Start --> End
"#;
        let result = parse(input).unwrap();
        assert!(matches!(result.direction, Direction::Bt));
    }

    #[test]
    fn test_parse_multiple_yes_edges() {
        // Condition node with multiple 'Yes' edges should fail
        let input = r#"flowchart TD
    Start --> A{x > 0?}
    A -->|Yes| B[println x]
    A -->|Yes| C[println y]
    A -->|No| End
    B --> End
    C --> End
"#;
        let result = parse(input);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("multiple 'Yes' edges"),
            "Expected error about multiple Yes edges, got: {}",
            err
        );
    }

    #[test]
    fn test_parse_multiple_no_edges() {
        // Condition node with multiple 'No' edges should fail
        let input = r#"flowchart TD
    Start --> A{x > 0?}
    A -->|Yes| B[println x]
    A -->|No| C[println y]
    A -->|No| End
    B --> End
    C --> End
"#;
        let result = parse(input);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("multiple 'No' edges"),
            "Expected error about multiple No edges, got: {}",
            err
        );
    }

    #[test]
    fn test_parse_invalid_identifier() {
        // Identifier starting with a digit should fail (pest grammar rejects this)
        let input = r#"flowchart TD
    Start --> 1abc[x = 1]
    1abc --> End
"#;
        let result = parse(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_empty_flowchart() {
        // Empty flowchart (no edges/nodes) should fail
        let input = r#"flowchart TD
"#;
        let result = parse(input);
        // An empty flowchart is technically valid syntax (0 lines),
        // but it's not useful. Let's verify it parses but produces empty nodes/edges.
        // Actually, let's check what happens - it might succeed with empty nodes.
        assert!(result.is_ok());
        let flowchart = result.unwrap();
        assert!(flowchart.nodes.is_empty());
        assert!(flowchart.edges.is_empty());
    }

    #[test]
    fn test_parse_missing_end_node() {
        // Flowchart without End node - this is valid syntax,
        // but the interpreter will fail at runtime.
        // Parser doesn't require End node presence.
        let input = r#"flowchart TD
    Start --> A[x = 1]
    A --> B[y = 2]
"#;
        let result = parse(input);
        // Parser allows this - End node validation is done at runtime
        assert!(result.is_ok());
        let flowchart = result.unwrap();
        let has_end = flowchart
            .nodes
            .iter()
            .any(|n| matches!(n, Node::End { .. }));
        assert!(!has_end, "Should not have End node");
    }

    #[test]
    fn test_parse_comment_at_start() {
        let input = r#"%% This is a comment
flowchart TD
    Start --> End
"#;
        let result = parse(input);
        assert!(result.is_ok(), "Should parse with comment at start");
    }

    #[test]
    fn test_parse_multiple_comment_lines_at_start() {
        let input = r#"%% First comment
%% Second comment
%% Third comment
flowchart TD
    Start --> End
"#;
        let result = parse(input);
        assert!(result.is_ok(), "Should parse with multiple comment lines");
    }

    #[test]
    fn test_parse_inline_comment() {
        let input = r#"flowchart TD
    Start --> A[x = 1] %% Inline comment
    A --> End
"#;
        let result = parse(input);
        assert!(result.is_ok(), "Should parse with inline comment");
        let flowchart = result.unwrap();
        assert_eq!(flowchart.edges.len(), 2);
    }

    #[test]
    fn test_parse_comment_after_direction() {
        let input = r#"flowchart TD %% direction comment
    Start --> End
"#;
        let result = parse(input);
        assert!(result.is_ok(), "Should parse with comment after direction");
    }

    #[test]
    fn test_parse_empty_comment() {
        let input = r#"%%
flowchart TD
    Start --> End
"#;
        let result = parse(input);
        assert!(result.is_ok(), "Should parse with empty comment");
    }

    #[test]
    fn test_parse_comment_with_special_chars() {
        let input = r#"%% Comment with special chars: !@#$%^&*(){}[]|
flowchart TD
    Start --> End
"#;
        let result = parse(input);
        assert!(result.is_ok(), "Should parse with special chars in comment");
    }

    #[test]
    fn test_parse_percent_in_string_not_comment() {
        // %% inside string literal should NOT be treated as comment
        let input = r#"flowchart TD
    Start --> A[println '%% not a comment']
    A --> End
"#;
        let result = parse(input);
        assert!(result.is_ok(), "Should parse with %% inside string literal");
        let flowchart = result.unwrap();
        assert_eq!(flowchart.edges.len(), 2);
    }

    #[test]
    fn test_parse_comment_between_edges() {
        let input = r#"flowchart TD
    Start --> A[x = 1]
    %% Comment on its own line
    A --> End
"#;
        let result = parse(input);
        assert!(
            result.is_ok(),
            "Should parse with comment between edge definitions"
        );
        let flowchart = result.unwrap();
        assert_eq!(flowchart.edges.len(), 2);
    }

    #[test]
    fn test_parse_comment_at_eof_no_trailing_newline() {
        let input = "flowchart TD\n    Start --> End\n%% Final comment with no newline";
        let result = parse(input);
        assert!(
            result.is_ok(),
            "Should parse with comment at EOF without trailing newline"
        );
    }

    #[test]
    fn test_parse_blank_and_comment_lines_at_start() {
        let input = r#"
%% Comment after blank line

%% Another comment
flowchart TD
    Start --> End
"#;
        let result = parse(input);
        assert!(
            result.is_ok(),
            "Should parse with mixed blank and comment lines at start"
        );
    }

    #[test]
    fn test_parse_comment_containing_double_percent() {
        let input = r#"%% This comment has %% another %% in it
flowchart TD
    Start --> End
"#;
        let result = parse(input);
        assert!(result.is_ok(), "Should parse with %% inside comment");
    }
}
