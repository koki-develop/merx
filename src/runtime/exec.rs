use crate::ast::Statement;

use super::env::Environment;
use super::error::RuntimeError;
use super::eval::{eval_expr, InputReader};

/// Trait for writing output (for testability).
pub trait OutputWriter {
    fn write_stdout(&mut self, s: &str);
    fn write_stderr(&mut self, s: &str);
}

/// Writes to standard output and standard error.
#[derive(Default)]
pub struct StdioWriter;

impl StdioWriter {
    pub fn new() -> Self {
        Self
    }
}

impl OutputWriter for StdioWriter {
    fn write_stdout(&mut self, s: &str) {
        println!("{}", s);
    }

    fn write_stderr(&mut self, s: &str) {
        eprintln!("{}", s);
    }
}

/// Executes a statement.
pub fn exec_statement<R: InputReader, W: OutputWriter>(
    stmt: &Statement,
    env: &mut Environment,
    input_reader: &mut R,
    output_writer: &mut W,
) -> Result<(), RuntimeError> {
    match stmt {
        Statement::Assign { variable, value } => {
            let val = eval_expr(value, env, input_reader)?;
            env.set(variable.clone(), val);
            Ok(())
        }
        Statement::Print { expr } => {
            let val = eval_expr(expr, env, input_reader)?;
            output_writer.write_stdout(&val.to_string());
            Ok(())
        }
        Statement::Error { message } => {
            let val = eval_expr(message, env, input_reader)?;
            output_writer.write_stderr(&val.to_string());
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Expr;

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

    /// Mock output writer for testing.
    struct MockOutputWriter {
        pub stdout: Vec<String>,
        pub stderr: Vec<String>,
    }

    impl MockOutputWriter {
        fn new() -> Self {
            Self {
                stdout: Vec::new(),
                stderr: Vec::new(),
            }
        }
    }

    impl OutputWriter for MockOutputWriter {
        fn write_stdout(&mut self, s: &str) {
            self.stdout.push(s.to_string());
        }

        fn write_stderr(&mut self, s: &str) {
            self.stderr.push(s.to_string());
        }
    }

    #[test]
    fn test_exec_assign() {
        let mut env = Environment::new();
        let mut input = MockInputReader::new(vec![]);
        let mut output = MockOutputWriter::new();

        let stmt = Statement::Assign {
            variable: "x".to_string(),
            value: Expr::IntLit { value: 42 },
        };

        exec_statement(&stmt, &mut env, &mut input, &mut output).unwrap();

        assert_eq!(
            env.get("x").unwrap(),
            &super::super::value::Value::Int(42)
        );
        assert!(output.stdout.is_empty());
        assert!(output.stderr.is_empty());
    }

    #[test]
    fn test_exec_print() {
        let mut env = Environment::new();
        let mut input = MockInputReader::new(vec![]);
        let mut output = MockOutputWriter::new();

        let stmt = Statement::Print {
            expr: Expr::StrLit {
                value: "hello".to_string(),
            },
        };

        exec_statement(&stmt, &mut env, &mut input, &mut output).unwrap();

        assert_eq!(output.stdout, vec!["hello"]);
        assert!(output.stderr.is_empty());
    }

    #[test]
    fn test_exec_print_int() {
        let mut env = Environment::new();
        let mut input = MockInputReader::new(vec![]);
        let mut output = MockOutputWriter::new();

        let stmt = Statement::Print {
            expr: Expr::IntLit { value: 42 },
        };

        exec_statement(&stmt, &mut env, &mut input, &mut output).unwrap();

        assert_eq!(output.stdout, vec!["42"]);
    }

    #[test]
    fn test_exec_error() {
        let mut env = Environment::new();
        let mut input = MockInputReader::new(vec![]);
        let mut output = MockOutputWriter::new();

        let stmt = Statement::Error {
            message: Expr::StrLit {
                value: "error message".to_string(),
            },
        };

        let result = exec_statement(&stmt, &mut env, &mut input, &mut output);

        assert!(result.is_ok());
        assert!(output.stdout.is_empty());
        assert_eq!(output.stderr, vec!["error message"]);
    }

    #[test]
    fn test_exec_assign_with_input() {
        let mut env = Environment::new();
        let mut input = MockInputReader::new(vec!["test input"]);
        let mut output = MockOutputWriter::new();

        let stmt = Statement::Assign {
            variable: "x".to_string(),
            value: Expr::Input,
        };

        exec_statement(&stmt, &mut env, &mut input, &mut output).unwrap();

        assert_eq!(
            env.get("x").unwrap(),
            &super::super::value::Value::Str("test input".to_string())
        );
    }
}
