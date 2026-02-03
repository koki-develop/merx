use pest::iterators::Pair;

use crate::ast::{BinaryOp, Expr, TypeName, UnaryOp};

use super::Rule;
use super::error::SyntaxError;

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
/// Returns [`SyntaxError`] if any sub-expression cannot be parsed.
///
/// # See Also
///
/// - [`build_expr_with_precedence`] - Handles precedence-based tree construction
/// - [`precedence`] - Defines operator precedence levels
pub(super) fn parse_expression(pair: Pair<Rule>) -> Result<Expr, SyntaxError> {
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
/// Returns [`SyntaxError`] if recursive parsing fails (propagated from callers).
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
) -> Result<Expr, SyntaxError> {
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
/// Returns [`SyntaxError`] if the inner cast expression cannot be parsed.
fn parse_unary_expr(pair: Pair<Rule>) -> Result<Expr, SyntaxError> {
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

    let mut expr =
        cast_expr.ok_or_else(|| SyntaxError::new("internal: expected cast_expr in unary_expr"))?;

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
/// Returns [`SyntaxError`] if the primary expression cannot be parsed.
fn parse_cast_expr(pair: Pair<Rule>) -> Result<Expr, SyntaxError> {
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

    let mut result =
        expr.ok_or_else(|| SyntaxError::new("internal: expected expr in cast_expr"))?;

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
/// Returns [`SyntaxError`] if a nested expression cannot be parsed.
fn parse_primary(pair: Pair<Rule>) -> Result<Expr, SyntaxError> {
    let inner = pair
        .into_inner()
        .next()
        .ok_or_else(|| SyntaxError::new("internal: expected inner in primary"))?;

    match inner.as_rule() {
        Rule::expression => parse_expression(inner),
        Rule::input_keyword => Ok(Expr::Input),
        Rule::bool_lit => Ok(Expr::BoolLit {
            value: inner.as_str() == "true",
        }),
        Rule::int_lit => {
            let s = inner.as_str();
            Ok(Expr::IntLit {
                value: s.parse::<i64>().map_err(|_| {
                    SyntaxError::new(format!("integer literal '{}' is out of range", s))
                })?,
            })
        }
        Rule::string_lit => {
            let s = inner.as_str();
            // Remove surrounding quotes
            let content = &s[1..s.len() - 1];
            Ok(Expr::StrLit {
                value: unescape_string(content)?,
            })
        }
        Rule::identifier => Ok(Expr::Variable {
            name: inner.as_str().to_string(),
        }),
        _ => unreachable!(),
    }
}

/// Processes escape sequences in a raw string extracted from between quotes.
///
/// Supports: `\\'`, `\\\\`, `\\n`, `\\t`, `\\r`, `\\0`, `\\xHH`.
/// The grammar guarantees that only valid escape sequences reach this function.
pub(super) fn unescape_string(raw: &str) -> Result<String, SyntaxError> {
    let mut result = String::with_capacity(raw.len());
    let mut chars = raw.chars();

    while let Some(c) = chars.next() {
        if c == '\\' {
            chars
                .next()
                .ok_or_else(|| SyntaxError::new("internal: truncated escape sequence"))?;
            let specifier = chars
                .next()
                .ok_or_else(|| SyntaxError::new("internal: truncated escape sequence"))?;
            match specifier {
                '\'' => result.push('\''),
                '\\' => {
                    chars
                        .next()
                        .ok_or_else(|| SyntaxError::new("internal: truncated backslash escape"))?;
                    result.push('\\');
                    result.push('\\');
                }
                'n' => result.push('\n'),
                't' => result.push('\t'),
                'r' => result.push('\r'),
                '0' => result.push('\0'),
                'x' => {
                    let h1 = chars
                        .next()
                        .and_then(|c| c.to_digit(16))
                        .ok_or_else(|| SyntaxError::new("internal: invalid hex escape"))?
                        as u8;
                    let h2 = chars
                        .next()
                        .and_then(|c| c.to_digit(16))
                        .ok_or_else(|| SyntaxError::new("internal: invalid hex escape"))?
                        as u8;
                    result.push((h1 * 16 + h2) as char);
                }
                _ => {
                    return Err(SyntaxError::new(
                        "internal: unexpected escape specifier in string",
                    ));
                }
            }
        } else {
            result.push(c);
        }
    }

    Ok(result)
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
