//! Expression evaluation.
//!
//! This module handles the evaluation of expressions ([`Expr`]) to produce
//! runtime values ([`Value`]).
//!
//! # Overview
//!
//! The evaluator recursively processes expression trees, handling:
//!
//! - **Literals**: Integer, string, and boolean constants
//! - **Variables**: Lookups in the environment
//! - **Input**: Reading from stdin
//! - **Unary operations**: Negation (`-`) and logical NOT (`!`)
//! - **Binary operations**: Arithmetic, comparison, equality, and logical operators
//! - **Type casts**: Explicit type conversions via `as`
//!
//! # Operator Semantics
//!
//! ## Arithmetic (`+`, `-`, `*`, `/`, `%`)
//!
//! - Operands must be integers
//! - Addition, subtraction, and multiplication use wrapping semantics
//! - Division and modulo check for zero divisor
//!
//! ## Comparison (`<`, `<=`, `>`, `>=`)
//!
//! - Operands must be integers
//! - Returns boolean
//!
//! ## Equality (`==`, `!=`)
//!
//! - Any types allowed (including cross-type comparison)
//! - Different types are never equal
//!
//! ## Logical (`&&`, `||`)
//!
//! - Operands must be booleans
//! - NOT short-circuiting (both sides always evaluated)
//!
//! # Input Handling
//!
//! The evaluator uses the [`InputReader`] trait for input operations,
//! allowing dependency injection for testing.

use std::io::{self, BufRead};

use crate::ast::{BinaryOp, Expr, TypeName, UnaryOp};

use super::env::Environment;
use super::error::RuntimeError;
use super::value::Value;

/// Abstraction for reading user input.
///
/// This trait allows the runtime to read input from different sources,
/// enabling both production use (stdin) and testing (mock input).
///
/// # Implementors
///
/// - [`StdinReader`] - Reads from standard input
/// - Test code can provide mock implementations
///
/// # Examples
///
/// ```ignore
/// struct MockInput(Vec<String>);
///
/// impl InputReader for MockInput {
///     fn read_line(&mut self) -> Result<String, RuntimeError> {
///         self.0.pop()
///             .ok_or_else(|| RuntimeError::IoError {
///                 message: "No more input".to_string()
///             })
///     }
/// }
/// ```
pub trait InputReader {
    /// Reads a single line of input.
    ///
    /// # Returns
    ///
    /// The input line with trailing newlines stripped.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError::IoError`] if reading fails.
    fn read_line(&mut self) -> Result<String, RuntimeError>;
}

/// Input reader that reads from standard input.
///
/// This is the default input reader used in production. It wraps a
/// [`BufRead`] implementation (typically buffered stdin).
///
/// # Type Parameter
///
/// - `R` - The underlying reader type, must implement [`BufRead`]
///
/// # Examples
///
/// ```
/// use merx::runtime::StdinReader;
///
/// // Create a reader for standard input
/// let reader = StdinReader::new();
/// ```
pub struct StdinReader<R: BufRead> {
    reader: R,
}

impl StdinReader<io::BufReader<io::Stdin>> {
    /// Creates a new stdin reader.
    ///
    /// This creates a buffered reader wrapping standard input.
    pub fn new() -> Self {
        Self {
            reader: io::BufReader::new(io::stdin()),
        }
    }
}

impl Default for StdinReader<io::BufReader<io::Stdin>> {
    fn default() -> Self {
        Self::new()
    }
}

impl<R: BufRead> InputReader for StdinReader<R> {
    /// Reads a line from the underlying reader.
    ///
    /// Trailing CR and LF characters are stripped from the result.
    fn read_line(&mut self) -> Result<String, RuntimeError> {
        let mut buf = String::new();
        self.reader
            .read_line(&mut buf)
            .map_err(|e| RuntimeError::IoError {
                message: e.to_string(),
            })?;
        // Trim trailing newlines
        Ok(buf.trim_end_matches(['\r', '\n']).to_string())
    }
}

/// Evaluates an expression to produce a runtime value.
///
/// This is the main entry point for expression evaluation. It recursively
/// evaluates the expression tree, performing operations and returning the
/// final value.
///
/// # Arguments
///
/// * `expr` - The expression AST node to evaluate
/// * `env` - The variable environment for lookups
/// * `input_reader` - The input source for `input` expressions
///
/// # Returns
///
/// The evaluated [`Value`], or a [`RuntimeError`] if evaluation fails.
///
/// # Errors
///
/// - [`RuntimeError::UndefinedVariable`] - Variable not in environment
/// - [`RuntimeError::TypeError`] - Operation applied to wrong type
/// - [`RuntimeError::CastError`] - Type cast failed
/// - [`RuntimeError::DivisionByZero`] - Division/modulo by zero
/// - [`RuntimeError::IoError`] - Input reading failed
///
/// # Examples
///
/// ```
/// use merx::ast::Expr;
/// use merx::runtime::{Environment, Value, StdinReader, eval_expr};
///
/// let mut env = Environment::new();
/// env.set("x".to_string(), Value::Int(5));
///
/// let mut input = StdinReader::new();
/// let expr = Expr::Variable { name: "x".to_string() };
///
/// // Note: This example won't actually compile in doc tests due to
/// // stdin not being mockable, but illustrates the API
/// ```
pub fn eval_expr<R: InputReader>(
    expr: &Expr,
    env: &Environment,
    input_reader: &mut R,
) -> Result<Value, RuntimeError> {
    match expr {
        Expr::IntLit { value } => Ok(Value::Int(*value)),
        Expr::StrLit { value } => Ok(Value::Str(value.clone())),
        Expr::BoolLit { value } => Ok(Value::Bool(*value)),

        Expr::Variable { name } => env.get(name).cloned(),

        Expr::Input => {
            let line = input_reader.read_line()?;
            Ok(Value::Str(line))
        }

        Expr::Unary { op, operand } => {
            let val = eval_expr(operand, env, input_reader)?;
            eval_unary(*op, val)
        }

        Expr::Binary { op, left, right } => {
            let left_val = eval_expr(left, env, input_reader)?;
            let right_val = eval_expr(right, env, input_reader)?;
            eval_binary(*op, left_val, right_val)
        }

        Expr::Cast { expr, target_type } => {
            let val = eval_expr(expr, env, input_reader)?;
            eval_cast(val, *target_type)
        }
    }
}

/// Evaluates a unary operation.
///
/// # Supported Operations
///
/// | Operator | Operand Type | Result Type | Description |
/// |----------|--------------|-------------|-------------|
/// | `!` | `bool` | `bool` | Logical NOT |
/// | `-` | `int` | `int` | Numeric negation |
///
/// # Arguments
///
/// * `op` - The unary operator
/// * `operand` - The operand value
///
/// # Returns
///
/// The result of applying the operator, or a type error.
///
/// # Errors
///
/// Returns [`RuntimeError::TypeError`] if the operand has the wrong type.
fn eval_unary(op: UnaryOp, operand: Value) -> Result<Value, RuntimeError> {
    match op {
        UnaryOp::Not => {
            let b = operand.as_bool().ok_or_else(|| RuntimeError::TypeError {
                expected: "bool",
                actual: operand.type_name(),
                operation: "logical NOT".to_string(),
            })?;
            Ok(Value::Bool(!b))
        }
        UnaryOp::Neg => {
            let n = operand.as_int().ok_or_else(|| RuntimeError::TypeError {
                expected: "int",
                actual: operand.type_name(),
                operation: "negation".to_string(),
            })?;
            Ok(Value::Int(-n))
        }
    }
}

/// Evaluates a binary operation.
///
/// # Operator Categories
///
/// ## Arithmetic (requires `int` operands, returns `int`)
/// - `+` Addition (wrapping)
/// - `-` Subtraction (wrapping)
/// - `*` Multiplication (wrapping)
/// - `/` Division (truncating toward zero)
/// - `%` Modulo (remainder)
///
/// ## Comparison (requires `int` operands, returns `bool`)
/// - `<` Less than
/// - `<=` Less than or equal
/// - `>` Greater than
/// - `>=` Greater than or equal
///
/// ## Equality (any types, returns `bool`)
/// - `==` Equal (types must match for true)
/// - `!=` Not equal
///
/// ## Logical (requires `bool` operands, returns `bool`)
/// - `&&` Logical AND (not short-circuiting)
/// - `||` Logical OR (not short-circuiting)
///
/// # Arguments
///
/// * `op` - The binary operator
/// * `left` - The left operand value
/// * `right` - The right operand value
///
/// # Returns
///
/// The result of applying the operator.
///
/// # Errors
///
/// - [`RuntimeError::TypeError`] - Operand has wrong type for operator
/// - [`RuntimeError::DivisionByZero`] - Division or modulo by zero
fn eval_binary(op: BinaryOp, left: Value, right: Value) -> Result<Value, RuntimeError> {
    match op {
        // Arithmetic (int only)
        BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
            let l = left.as_int().ok_or_else(|| RuntimeError::TypeError {
                expected: "int",
                actual: left.type_name(),
                operation: format!("{:?}", op),
            })?;
            let r = right.as_int().ok_or_else(|| RuntimeError::TypeError {
                expected: "int",
                actual: right.type_name(),
                operation: format!("{:?}", op),
            })?;
            let result = match op {
                BinaryOp::Add => l.wrapping_add(r),
                BinaryOp::Sub => l.wrapping_sub(r),
                BinaryOp::Mul => l.wrapping_mul(r),
                BinaryOp::Div => {
                    if r == 0 {
                        return Err(RuntimeError::DivisionByZero);
                    }
                    l / r
                }
                BinaryOp::Mod => {
                    if r == 0 {
                        return Err(RuntimeError::DivisionByZero);
                    }
                    l % r
                }
                _ => unreachable!(),
            };
            Ok(Value::Int(result))
        }

        // Equality (all types)
        BinaryOp::Eq => Ok(Value::Bool(left == right)),
        BinaryOp::Ne => Ok(Value::Bool(left != right)),

        // Comparison (int only)
        BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
            let l = left.as_int().ok_or_else(|| RuntimeError::TypeError {
                expected: "int",
                actual: left.type_name(),
                operation: format!("{:?}", op),
            })?;
            let r = right.as_int().ok_or_else(|| RuntimeError::TypeError {
                expected: "int",
                actual: right.type_name(),
                operation: format!("{:?}", op),
            })?;
            let result = match op {
                BinaryOp::Lt => l < r,
                BinaryOp::Le => l <= r,
                BinaryOp::Gt => l > r,
                BinaryOp::Ge => l >= r,
                _ => unreachable!(),
            };
            Ok(Value::Bool(result))
        }

        // Logical (bool only)
        BinaryOp::And | BinaryOp::Or => {
            let l = left.as_bool().ok_or_else(|| RuntimeError::TypeError {
                expected: "bool",
                actual: left.type_name(),
                operation: format!("{:?}", op),
            })?;
            let r = right.as_bool().ok_or_else(|| RuntimeError::TypeError {
                expected: "bool",
                actual: right.type_name(),
                operation: format!("{:?}", op),
            })?;
            let result = match op {
                BinaryOp::And => l && r,
                BinaryOp::Or => l || r,
                _ => unreachable!(),
            };
            Ok(Value::Bool(result))
        }
    }
}

/// Evaluates a type cast expression.
///
/// # Supported Casts
///
/// | From | To | Behavior |
/// |------|-----|----------|
/// | `int` | `int` | Identity (no-op) |
/// | `str` | `int` | Parse as decimal integer |
/// | `bool` | `int` | **Error** - not supported |
/// | `int` | `str` | Decimal string representation |
/// | `str` | `str` | Identity (no-op) |
/// | `bool` | `str` | `"true"` or `"false"` |
///
/// Note: Casting to `bool` is not supported in the language.
///
/// # Arguments
///
/// * `val` - The value to cast
/// * `target` - The target type
///
/// # Returns
///
/// The cast value.
///
/// # Errors
///
/// Returns [`RuntimeError::CastError`] if the cast is not supported
/// or if string-to-int parsing fails.
fn eval_cast(val: Value, target: TypeName) -> Result<Value, RuntimeError> {
    match target {
        TypeName::Int => match val {
            Value::Int(n) => Ok(Value::Int(n)),
            Value::Str(ref s) => {
                s.parse::<i64>()
                    .map(Value::Int)
                    .map_err(|_| RuntimeError::CastError {
                        from_type: "str",
                        to_type: "int",
                        value: s.clone(),
                    })
            }
            Value::Bool(_) => Err(RuntimeError::CastError {
                from_type: "bool",
                to_type: "int",
                value: val.to_string(),
            }),
        },
        TypeName::Str => Ok(Value::Str(val.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mock input reader for testing.
    struct MockInputReader {
        lines: Vec<String>,
        index: usize,
    }

    impl MockInputReader {
        fn new(lines: Vec<&str>) -> Self {
            Self {
                lines: lines.into_iter().map(|s| s.to_string()).collect(),
                index: 0,
            }
        }
    }

    impl InputReader for MockInputReader {
        fn read_line(&mut self) -> Result<String, RuntimeError> {
            if self.index < self.lines.len() {
                let line = self.lines[self.index].clone();
                self.index += 1;
                Ok(line)
            } else {
                Err(RuntimeError::IoError {
                    message: "No more input".to_string(),
                })
            }
        }
    }

    #[test]
    fn test_eval_int_literal() {
        let env = Environment::new();
        let mut input = MockInputReader::new(vec![]);
        let expr = Expr::IntLit { value: 42 };
        let result = eval_expr(&expr, &env, &mut input).unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_eval_str_literal() {
        let env = Environment::new();
        let mut input = MockInputReader::new(vec![]);
        let expr = Expr::StrLit {
            value: "hello".to_string(),
        };
        let result = eval_expr(&expr, &env, &mut input).unwrap();
        assert_eq!(result, Value::Str("hello".to_string()));
    }

    #[test]
    fn test_eval_bool_literal() {
        let env = Environment::new();
        let mut input = MockInputReader::new(vec![]);
        let expr = Expr::BoolLit { value: true };
        let result = eval_expr(&expr, &env, &mut input).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_eval_variable() {
        let mut env = Environment::new();
        env.set("x".to_string(), Value::Int(10));
        let mut input = MockInputReader::new(vec![]);
        let expr = Expr::Variable {
            name: "x".to_string(),
        };
        let result = eval_expr(&expr, &env, &mut input).unwrap();
        assert_eq!(result, Value::Int(10));
    }

    #[test]
    fn test_eval_undefined_variable() {
        let env = Environment::new();
        let mut input = MockInputReader::new(vec![]);
        let expr = Expr::Variable {
            name: "x".to_string(),
        };
        let result = eval_expr(&expr, &env, &mut input);
        assert!(matches!(
            result,
            Err(RuntimeError::UndefinedVariable { name }) if name == "x"
        ));
    }

    #[test]
    fn test_eval_input() {
        let env = Environment::new();
        let mut input = MockInputReader::new(vec!["hello"]);
        let expr = Expr::Input;
        let result = eval_expr(&expr, &env, &mut input).unwrap();
        assert_eq!(result, Value::Str("hello".to_string()));
    }

    #[test]
    fn test_eval_unary_not() {
        let env = Environment::new();
        let mut input = MockInputReader::new(vec![]);
        let expr = Expr::Unary {
            op: UnaryOp::Not,
            operand: Box::new(Expr::BoolLit { value: true }),
        };
        let result = eval_expr(&expr, &env, &mut input).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_eval_unary_neg() {
        let env = Environment::new();
        let mut input = MockInputReader::new(vec![]);
        let expr = Expr::Unary {
            op: UnaryOp::Neg,
            operand: Box::new(Expr::IntLit { value: 42 }),
        };
        let result = eval_expr(&expr, &env, &mut input).unwrap();
        assert_eq!(result, Value::Int(-42));
    }

    #[test]
    fn test_eval_binary_add() {
        let env = Environment::new();
        let mut input = MockInputReader::new(vec![]);
        let expr = Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expr::IntLit { value: 1 }),
            right: Box::new(Expr::IntLit { value: 2 }),
        };
        let result = eval_expr(&expr, &env, &mut input).unwrap();
        assert_eq!(result, Value::Int(3));
    }

    #[test]
    fn test_eval_binary_sub() {
        let env = Environment::new();
        let mut input = MockInputReader::new(vec![]);
        let expr = Expr::Binary {
            op: BinaryOp::Sub,
            left: Box::new(Expr::IntLit { value: 5 }),
            right: Box::new(Expr::IntLit { value: 3 }),
        };
        let result = eval_expr(&expr, &env, &mut input).unwrap();
        assert_eq!(result, Value::Int(2));
    }

    #[test]
    fn test_eval_binary_mul() {
        let env = Environment::new();
        let mut input = MockInputReader::new(vec![]);
        let expr = Expr::Binary {
            op: BinaryOp::Mul,
            left: Box::new(Expr::IntLit { value: 3 }),
            right: Box::new(Expr::IntLit { value: 4 }),
        };
        let result = eval_expr(&expr, &env, &mut input).unwrap();
        assert_eq!(result, Value::Int(12));
    }

    #[test]
    fn test_eval_binary_div() {
        let env = Environment::new();
        let mut input = MockInputReader::new(vec![]);
        let expr = Expr::Binary {
            op: BinaryOp::Div,
            left: Box::new(Expr::IntLit { value: 10 }),
            right: Box::new(Expr::IntLit { value: 3 }),
        };
        let result = eval_expr(&expr, &env, &mut input).unwrap();
        assert_eq!(result, Value::Int(3));
    }

    #[test]
    fn test_eval_binary_mod() {
        let env = Environment::new();
        let mut input = MockInputReader::new(vec![]);
        let expr = Expr::Binary {
            op: BinaryOp::Mod,
            left: Box::new(Expr::IntLit { value: 10 }),
            right: Box::new(Expr::IntLit { value: 3 }),
        };
        let result = eval_expr(&expr, &env, &mut input).unwrap();
        assert_eq!(result, Value::Int(1));
    }

    #[test]
    fn test_eval_division_by_zero() {
        let env = Environment::new();
        let mut input = MockInputReader::new(vec![]);
        let expr = Expr::Binary {
            op: BinaryOp::Div,
            left: Box::new(Expr::IntLit { value: 10 }),
            right: Box::new(Expr::IntLit { value: 0 }),
        };
        let result = eval_expr(&expr, &env, &mut input);
        assert!(matches!(result, Err(RuntimeError::DivisionByZero)));
    }

    #[test]
    fn test_eval_mod_by_zero() {
        let env = Environment::new();
        let mut input = MockInputReader::new(vec![]);
        let expr = Expr::Binary {
            op: BinaryOp::Mod,
            left: Box::new(Expr::IntLit { value: 10 }),
            right: Box::new(Expr::IntLit { value: 0 }),
        };
        let result = eval_expr(&expr, &env, &mut input);
        assert!(matches!(result, Err(RuntimeError::DivisionByZero)));
    }

    #[test]
    fn test_eval_comparison() {
        let env = Environment::new();
        let mut input = MockInputReader::new(vec![]);

        let expr_lt = Expr::Binary {
            op: BinaryOp::Lt,
            left: Box::new(Expr::IntLit { value: 1 }),
            right: Box::new(Expr::IntLit { value: 2 }),
        };
        assert_eq!(
            eval_expr(&expr_lt, &env, &mut input).unwrap(),
            Value::Bool(true)
        );

        let expr_le = Expr::Binary {
            op: BinaryOp::Le,
            left: Box::new(Expr::IntLit { value: 2 }),
            right: Box::new(Expr::IntLit { value: 2 }),
        };
        assert_eq!(
            eval_expr(&expr_le, &env, &mut input).unwrap(),
            Value::Bool(true)
        );

        let expr_gt = Expr::Binary {
            op: BinaryOp::Gt,
            left: Box::new(Expr::IntLit { value: 3 }),
            right: Box::new(Expr::IntLit { value: 2 }),
        };
        assert_eq!(
            eval_expr(&expr_gt, &env, &mut input).unwrap(),
            Value::Bool(true)
        );

        let expr_ge = Expr::Binary {
            op: BinaryOp::Ge,
            left: Box::new(Expr::IntLit { value: 2 }),
            right: Box::new(Expr::IntLit { value: 2 }),
        };
        assert_eq!(
            eval_expr(&expr_ge, &env, &mut input).unwrap(),
            Value::Bool(true)
        );
    }

    #[test]
    fn test_eval_equality() {
        let env = Environment::new();
        let mut input = MockInputReader::new(vec![]);

        let expr_eq = Expr::Binary {
            op: BinaryOp::Eq,
            left: Box::new(Expr::IntLit { value: 1 }),
            right: Box::new(Expr::IntLit { value: 1 }),
        };
        assert_eq!(
            eval_expr(&expr_eq, &env, &mut input).unwrap(),
            Value::Bool(true)
        );

        let expr_ne = Expr::Binary {
            op: BinaryOp::Ne,
            left: Box::new(Expr::IntLit { value: 1 }),
            right: Box::new(Expr::IntLit { value: 2 }),
        };
        assert_eq!(
            eval_expr(&expr_ne, &env, &mut input).unwrap(),
            Value::Bool(true)
        );

        // Comparison of different types
        let expr_eq_diff_type = Expr::Binary {
            op: BinaryOp::Eq,
            left: Box::new(Expr::IntLit { value: 1 }),
            right: Box::new(Expr::StrLit {
                value: "1".to_string(),
            }),
        };
        assert_eq!(
            eval_expr(&expr_eq_diff_type, &env, &mut input).unwrap(),
            Value::Bool(false)
        );
    }

    #[test]
    fn test_eval_logical() {
        let env = Environment::new();
        let mut input = MockInputReader::new(vec![]);

        let expr_and = Expr::Binary {
            op: BinaryOp::And,
            left: Box::new(Expr::BoolLit { value: true }),
            right: Box::new(Expr::BoolLit { value: false }),
        };
        assert_eq!(
            eval_expr(&expr_and, &env, &mut input).unwrap(),
            Value::Bool(false)
        );

        let expr_or = Expr::Binary {
            op: BinaryOp::Or,
            left: Box::new(Expr::BoolLit { value: true }),
            right: Box::new(Expr::BoolLit { value: false }),
        };
        assert_eq!(
            eval_expr(&expr_or, &env, &mut input).unwrap(),
            Value::Bool(true)
        );
    }

    #[test]
    fn test_eval_cast_str_to_int() {
        let env = Environment::new();
        let mut input = MockInputReader::new(vec![]);
        let expr = Expr::Cast {
            expr: Box::new(Expr::StrLit {
                value: "123".to_string(),
            }),
            target_type: TypeName::Int,
        };
        let result = eval_expr(&expr, &env, &mut input).unwrap();
        assert_eq!(result, Value::Int(123));
    }

    #[test]
    fn test_eval_cast_str_to_int_invalid() {
        let env = Environment::new();
        let mut input = MockInputReader::new(vec![]);
        let expr = Expr::Cast {
            expr: Box::new(Expr::StrLit {
                value: "abc".to_string(),
            }),
            target_type: TypeName::Int,
        };
        let result = eval_expr(&expr, &env, &mut input);
        assert!(matches!(result, Err(RuntimeError::CastError { .. })));
    }

    #[test]
    fn test_eval_cast_int_to_str() {
        let env = Environment::new();
        let mut input = MockInputReader::new(vec![]);
        let expr = Expr::Cast {
            expr: Box::new(Expr::IntLit { value: 42 }),
            target_type: TypeName::Str,
        };
        let result = eval_expr(&expr, &env, &mut input).unwrap();
        assert_eq!(result, Value::Str("42".to_string()));
    }

    #[test]
    fn test_eval_cast_bool_to_str() {
        let env = Environment::new();
        let mut input = MockInputReader::new(vec![]);
        let expr = Expr::Cast {
            expr: Box::new(Expr::BoolLit { value: true }),
            target_type: TypeName::Str,
        };
        let result = eval_expr(&expr, &env, &mut input).unwrap();
        assert_eq!(result, Value::Str("true".to_string()));
    }

    #[test]
    fn test_eval_type_error_arithmetic() {
        let env = Environment::new();
        let mut input = MockInputReader::new(vec![]);
        let expr = Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expr::IntLit { value: 1 }),
            right: Box::new(Expr::StrLit {
                value: "2".to_string(),
            }),
        };
        let result = eval_expr(&expr, &env, &mut input);
        assert!(matches!(result, Err(RuntimeError::TypeError { .. })));
    }

    use crate::ast::{Node, Statement};
    use crate::parser::parse;

    /// Helper function to parse an expression from an assignment statement and evaluate it.
    fn parse_and_eval(expr_str: &str) -> Value {
        let input = format!(
            r#"flowchart TD
    Start --> A[result = {}]
    A --> End
"#,
            expr_str
        );
        let flowchart = parse(&input).expect("Failed to parse");
        let process_node = flowchart
            .nodes
            .iter()
            .find(|n| matches!(n, Node::Process { .. }))
            .expect("Process node not found");
        let expr = match process_node {
            Node::Process { statements, .. } => match &statements[0] {
                Statement::Assign { value, .. } => value.clone(),
                _ => panic!("Expected Assign statement"),
            },
            _ => unreachable!(),
        };

        let env = Environment::new();
        let mut mock_input = MockInputReader::new(vec![]);
        eval_expr(&expr, &env, &mut mock_input).expect("Evaluation failed")
    }

    /// Helper function to parse a condition expression and evaluate it.
    fn parse_and_eval_condition(expr_str: &str) -> Value {
        let input = format!(
            r#"flowchart TD
    Start --> A{{{}?}}
    A -->|Yes| End
    A -->|No| End
"#,
            expr_str
        );
        let flowchart = parse(&input).expect("Failed to parse");
        let condition_node = flowchart
            .nodes
            .iter()
            .find(|n| matches!(n, Node::Condition { .. }))
            .expect("Condition node not found");
        let expr = match condition_node {
            Node::Condition { condition, .. } => condition.clone(),
            _ => unreachable!(),
        };

        let env = Environment::new();
        let mut mock_input = MockInputReader::new(vec![]);
        eval_expr(&expr, &env, &mut mock_input).expect("Evaluation failed")
    }

    #[test]
    fn test_left_associativity_subtraction() {
        // 1 - 2 - 3 should be (1 - 2) - 3 = -1 - 3 = -4
        let result = parse_and_eval("1 - 2 - 3");
        assert_eq!(result, Value::Int(-4));
    }

    #[test]
    fn test_left_associativity_division() {
        // 12 / 3 / 2 should be (12 / 3) / 2 = 4 / 2 = 2
        let result = parse_and_eval("12 / 3 / 2");
        assert_eq!(result, Value::Int(2));
    }

    #[test]
    fn test_precedence_add_mul() {
        // 1 + 2 * 3 should be 1 + (2 * 3) = 1 + 6 = 7
        let result = parse_and_eval("1 + 2 * 3");
        assert_eq!(result, Value::Int(7));
    }

    #[test]
    fn test_precedence_comparison_logical() {
        // true && 1 < 2 should be true && (1 < 2) = true && true = true
        let result = parse_and_eval_condition("true && 1 < 2");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_complex_expression() {
        // (1 + 2) * 3 - 4 / 2 should be ((1 + 2) * 3) - (4 / 2) = 9 - 2 = 7
        let result = parse_and_eval("(1 + 2) * 3 - 4 / 2");
        assert_eq!(result, Value::Int(7));
    }

    #[test]
    fn test_int_max_value() {
        let env = Environment::new();
        let mut input = MockInputReader::new(vec![]);
        let expr = Expr::IntLit { value: i64::MAX };
        let result = eval_expr(&expr, &env, &mut input).unwrap();
        assert_eq!(result, Value::Int(i64::MAX));
        assert_eq!(result, Value::Int(9223372036854775807));
    }

    #[test]
    fn test_int_min_value() {
        let env = Environment::new();
        let mut input = MockInputReader::new(vec![]);
        let expr = Expr::IntLit { value: i64::MIN };
        let result = eval_expr(&expr, &env, &mut input).unwrap();
        assert_eq!(result, Value::Int(i64::MIN));
        assert_eq!(result, Value::Int(-9223372036854775808));
    }

    #[test]
    fn test_int_overflow_add() {
        // i64::MAX + 1 should wrap to i64::MIN (wrapping_add behavior)
        let env = Environment::new();
        let mut input = MockInputReader::new(vec![]);
        let expr = Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expr::IntLit { value: i64::MAX }),
            right: Box::new(Expr::IntLit { value: 1 }),
        };
        let result = eval_expr(&expr, &env, &mut input).unwrap();
        assert_eq!(result, Value::Int(i64::MIN));
    }

    #[test]
    fn test_int_overflow_mul() {
        // i64::MAX * 2 should wrap (wrapping_mul behavior)
        // i64::MAX = 9223372036854775807
        // i64::MAX * 2 wraps to -2
        let env = Environment::new();
        let mut input = MockInputReader::new(vec![]);
        let expr = Expr::Binary {
            op: BinaryOp::Mul,
            left: Box::new(Expr::IntLit { value: i64::MAX }),
            right: Box::new(Expr::IntLit { value: 2 }),
        };
        let result = eval_expr(&expr, &env, &mut input).unwrap();
        assert_eq!(result, Value::Int(i64::MAX.wrapping_mul(2)));
        assert_eq!(result, Value::Int(-2));
    }

    #[test]
    fn test_negative_mod() {
        // -10 % 3 should be -1 (Rust's remainder semantics)
        let env = Environment::new();
        let mut input = MockInputReader::new(vec![]);
        let expr = Expr::Binary {
            op: BinaryOp::Mod,
            left: Box::new(Expr::IntLit { value: -10 }),
            right: Box::new(Expr::IntLit { value: 3 }),
        };
        let result = eval_expr(&expr, &env, &mut input).unwrap();
        assert_eq!(result, Value::Int(-1));
    }

    #[test]
    fn test_cast_max_int_string() {
        // "9223372036854775807" as int should succeed with i64::MAX
        let env = Environment::new();
        let mut input = MockInputReader::new(vec![]);
        let expr = Expr::Cast {
            expr: Box::new(Expr::StrLit {
                value: "9223372036854775807".to_string(),
            }),
            target_type: TypeName::Int,
        };
        let result = eval_expr(&expr, &env, &mut input).unwrap();
        assert_eq!(result, Value::Int(i64::MAX));
    }

    #[test]
    fn test_cast_overflow_string() {
        // "9223372036854775808" as int should fail (overflow)
        let env = Environment::new();
        let mut input = MockInputReader::new(vec![]);
        let expr = Expr::Cast {
            expr: Box::new(Expr::StrLit {
                value: "9223372036854775808".to_string(),
            }),
            target_type: TypeName::Int,
        };
        let result = eval_expr(&expr, &env, &mut input);
        assert!(matches!(
            result,
            Err(RuntimeError::CastError {
                from_type: "str",
                to_type: "int",
                ..
            })
        ));
    }

    #[test]
    fn test_cast_negative_string() {
        // "-42" as int should succeed with -42
        let env = Environment::new();
        let mut input = MockInputReader::new(vec![]);
        let expr = Expr::Cast {
            expr: Box::new(Expr::StrLit {
                value: "-42".to_string(),
            }),
            target_type: TypeName::Int,
        };
        let result = eval_expr(&expr, &env, &mut input).unwrap();
        assert_eq!(result, Value::Int(-42));
    }

    #[test]
    fn test_cast_empty_string() {
        // "" as int should fail
        let env = Environment::new();
        let mut input = MockInputReader::new(vec![]);
        let expr = Expr::Cast {
            expr: Box::new(Expr::StrLit {
                value: "".to_string(),
            }),
            target_type: TypeName::Int,
        };
        let result = eval_expr(&expr, &env, &mut input);
        assert!(matches!(
            result,
            Err(RuntimeError::CastError {
                from_type: "str",
                to_type: "int",
                ..
            })
        ));
    }

    #[test]
    fn test_input_eof_mock_reader() {
        // MockInputReader returns IoError when no more input is available
        let env = Environment::new();
        let mut input = MockInputReader::new(vec![]); // No input available
        let expr = Expr::Input;

        let result = eval_expr(&expr, &env, &mut input);
        assert!(matches!(
            result,
            Err(RuntimeError::IoError { message }) if message == "No more input"
        ));
    }

    #[test]
    fn test_input_eof_stdin_reader() {
        // StdinReader returns empty string at EOF (not an error)
        // This tests the actual StdinReader behavior with an empty buffer
        use std::io::Cursor;

        let env = Environment::new();
        let empty_buffer = Cursor::new(Vec::<u8>::new());
        let mut input = StdinReader {
            reader: empty_buffer,
        };
        let expr = Expr::Input;

        let result = eval_expr(&expr, &env, &mut input).unwrap();
        // At EOF, read_line returns Ok("") (empty string after trimming)
        assert_eq!(result, Value::Str("".to_string()));
    }

    #[test]
    fn test_input_empty_line() {
        // Empty line (just newline) should return empty string
        use std::io::Cursor;

        let env = Environment::new();
        let buffer = Cursor::new(b"\n".to_vec());
        let mut input = StdinReader { reader: buffer };
        let expr = Expr::Input;

        let result = eval_expr(&expr, &env, &mut input).unwrap();
        assert_eq!(result, Value::Str("".to_string()));
    }

    #[test]
    fn test_input_empty_line_crlf() {
        // Empty line with CRLF should return empty string
        use std::io::Cursor;

        let env = Environment::new();
        let buffer = Cursor::new(b"\r\n".to_vec());
        let mut input = StdinReader { reader: buffer };
        let expr = Expr::Input;

        let result = eval_expr(&expr, &env, &mut input).unwrap();
        assert_eq!(result, Value::Str("".to_string()));
    }

    #[test]
    fn test_input_whitespace_spaces() {
        // Line with only spaces should preserve those spaces
        use std::io::Cursor;

        let env = Environment::new();
        let buffer = Cursor::new(b"   \n".to_vec());
        let mut input = StdinReader { reader: buffer };
        let expr = Expr::Input;

        let result = eval_expr(&expr, &env, &mut input).unwrap();
        // Only trailing \r and \n are trimmed, spaces are preserved
        assert_eq!(result, Value::Str("   ".to_string()));
    }

    #[test]
    fn test_input_whitespace_tabs() {
        // Line with only tabs should preserve those tabs
        use std::io::Cursor;

        let env = Environment::new();
        let buffer = Cursor::new(b"\t\t\n".to_vec());
        let mut input = StdinReader { reader: buffer };
        let expr = Expr::Input;

        let result = eval_expr(&expr, &env, &mut input).unwrap();
        // Only trailing \r and \n are trimmed, tabs are preserved
        assert_eq!(result, Value::Str("\t\t".to_string()));
    }

    #[test]
    fn test_input_whitespace_mixed() {
        // Line with mixed whitespace (spaces and tabs) should preserve them
        use std::io::Cursor;

        let env = Environment::new();
        let buffer = Cursor::new(b" \t \t \n".to_vec());
        let mut input = StdinReader { reader: buffer };
        let expr = Expr::Input;

        let result = eval_expr(&expr, &env, &mut input).unwrap();
        assert_eq!(result, Value::Str(" \t \t ".to_string()));
    }

    #[test]
    fn test_input_multiple_reads() {
        // Multiple reads should return lines in order
        use std::io::Cursor;

        let env = Environment::new();
        let buffer = Cursor::new(b"first\nsecond\nthird\n".to_vec());
        let mut input = StdinReader { reader: buffer };

        let expr = Expr::Input;

        let result1 = eval_expr(&expr, &env, &mut input).unwrap();
        assert_eq!(result1, Value::Str("first".to_string()));

        let result2 = eval_expr(&expr, &env, &mut input).unwrap();
        assert_eq!(result2, Value::Str("second".to_string()));

        let result3 = eval_expr(&expr, &env, &mut input).unwrap();
        assert_eq!(result3, Value::Str("third".to_string()));

        // Fourth read should return empty string (EOF)
        let result4 = eval_expr(&expr, &env, &mut input).unwrap();
        assert_eq!(result4, Value::Str("".to_string()));
    }

    #[test]
    fn test_input_no_trailing_newline() {
        // Line without trailing newline (last line of file)
        use std::io::Cursor;

        let env = Environment::new();
        let buffer = Cursor::new(b"no newline at end".to_vec());
        let mut input = StdinReader { reader: buffer };
        let expr = Expr::Input;

        let result = eval_expr(&expr, &env, &mut input).unwrap();
        assert_eq!(result, Value::Str("no newline at end".to_string()));
    }
}
