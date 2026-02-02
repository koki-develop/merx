use std::io::{self, BufRead};

use crate::ast::{BinaryOp, Expr, TypeName, UnaryOp};

use super::env::Environment;
use super::error::RuntimeError;
use super::value::Value;

/// Trait for reading input (for testability).
pub trait InputReader {
    fn read_line(&mut self) -> Result<String, RuntimeError>;
}

/// Reads from standard input.
pub struct StdinReader<R: BufRead> {
    reader: R,
}

impl StdinReader<io::BufReader<io::Stdin>> {
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

/// Evaluates an expression and returns a Value.
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

/// Evaluates a type cast.
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
}
