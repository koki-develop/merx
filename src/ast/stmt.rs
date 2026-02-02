//! Statement types for executable actions.
//!
//! Statements are the basic units of execution within process nodes. Unlike
//! expressions which produce values, statements perform actions like outputting
//! text, signaling errors, or storing values in variables.

use serde::Serialize;

use super::Expr;

/// An executable statement within a process node.
///
/// Statements are executed sequentially within a [`Process`](super::Node::Process)
/// node. Multiple statements can be separated by semicolons.
///
/// # Statement Types
///
/// | Variant | Mermaid Syntax | Description |
/// |---------|----------------|-------------|
/// | [`Assign`](Statement::Assign) | `x = expr` | Store value in variable |
/// | [`Print`](Statement::Print) | `println expr` | Write to stdout with newline |
/// | [`PrintNoNewline`](Statement::PrintNoNewline) | `print expr` | Write to stdout without newline |
/// | [`Error`](Statement::Error) | `error expr` | Write to stderr and terminate |
///
/// # Examples
///
/// ```text
/// A[x = 5; println x]           // Assign then print
/// B[name = input; println name] // Read input and display
/// C[error 'Invalid input']      // Signal an error
/// ```
///
/// # Serialization
///
/// Uses tagged enum serialization with `"type"` discriminator:
///
/// ```json
/// { "type": "assign", "variable": "x", "value": {...} }
/// { "type": "print", "expr": {...} }
/// { "type": "error", "message": {...} }
/// ```
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Statement {
    /// Assign a value to a variable.
    ///
    /// Creates a new variable or updates an existing one. Variables are
    /// dynamically typed; the same variable can hold different types at
    /// different times.
    ///
    /// # Mermaid Syntax
    ///
    /// ```text
    /// x = 42
    /// name = 'Alice'
    /// value = input as int
    /// sum = a + b
    /// ```
    ///
    /// # Scope
    ///
    /// All variables are global within a flowchart execution. A variable
    /// assigned in one node is accessible in all subsequent nodes.
    Assign {
        /// The name of the variable to assign to.
        variable: String,

        /// The expression whose value will be stored.
        value: Expr,
    },

    /// Print a value to standard output.
    ///
    /// Evaluates the expression and writes the result to stdout, followed
    /// by a newline. All value types can be printed.
    ///
    /// # Mermaid Syntax
    ///
    /// ```text
    /// println 'Hello, World!'
    /// println x
    /// println a + b
    /// ```
    ///
    /// # Output Format
    ///
    /// - Integers: Decimal representation (e.g., `42`)
    /// - Strings: The string value as-is (e.g., `Hello`)
    /// - Booleans: `true` or `false`
    Print {
        /// The expression to evaluate and print.
        expr: Expr,
    },

    /// Print a value to standard output without a trailing newline.
    ///
    /// Evaluates the expression and writes the result to stdout without
    /// appending a newline. This is useful for building output incrementally.
    ///
    /// # Mermaid Syntax
    ///
    /// ```text
    /// print 'Hello, '
    /// print x
    /// ```
    ///
    /// # Output Format
    ///
    /// Same as [`Print`](Statement::Print), but without the trailing newline.
    PrintNoNewline {
        /// The expression to evaluate and print.
        expr: Expr,
    },

    /// Print an error message to stderr.
    ///
    /// Evaluates the expression and writes the result to stderr. Unlike `print`,
    /// the output goes to stderr instead of stdout.
    ///
    /// # Mermaid Syntax
    ///
    /// ```text
    /// error 'Something went wrong'
    /// error 'Invalid value: ' + x as str
    /// ```
    ///
    /// # Behavior
    ///
    /// Execution continues after the error message is printed. To terminate
    /// the program, use a condition branch leading to `End`.
    Error {
        /// The expression to evaluate and display as an error message.
        message: Expr,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{BinaryOp, TypeName};

    #[test]
    fn test_statement_serialize_print() {
        let stmt = Statement::Print {
            expr: Expr::StrLit {
                value: "hello".to_string(),
            },
        };
        let json = serde_json::to_value(&stmt).unwrap();

        assert_eq!(json["type"], "print");
        assert_eq!(json["expr"]["type"], "str_lit");
        assert_eq!(json["expr"]["value"], "hello");
    }

    #[test]
    fn test_statement_serialize_print_complex_expr() {
        let stmt = Statement::Print {
            expr: Expr::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::IntLit { value: 1 }),
                right: Box::new(Expr::IntLit { value: 2 }),
            },
        };
        let json = serde_json::to_value(&stmt).unwrap();

        assert_eq!(json["type"], "print");
        assert_eq!(json["expr"]["type"], "binary");
        assert_eq!(json["expr"]["op"], "add");
    }

    #[test]
    fn test_statement_serialize_print_no_newline() {
        let stmt = Statement::PrintNoNewline {
            expr: Expr::StrLit {
                value: "hello".to_string(),
            },
        };
        let json = serde_json::to_value(&stmt).unwrap();

        assert_eq!(json["type"], "print_no_newline");
        assert_eq!(json["expr"]["type"], "str_lit");
        assert_eq!(json["expr"]["value"], "hello");
    }

    #[test]
    fn test_statement_serialize_error() {
        let stmt = Statement::Error {
            message: Expr::StrLit {
                value: "Something went wrong".to_string(),
            },
        };
        let json = serde_json::to_value(&stmt).unwrap();

        assert_eq!(json["type"], "error");
        assert_eq!(json["message"]["type"], "str_lit");
        assert_eq!(json["message"]["value"], "Something went wrong");
    }

    #[test]
    fn test_statement_serialize_error_with_variable() {
        let stmt = Statement::Error {
            message: Expr::Variable {
                name: "err_msg".to_string(),
            },
        };
        let json = serde_json::to_value(&stmt).unwrap();

        assert_eq!(json["type"], "error");
        assert_eq!(json["message"]["type"], "variable");
        assert_eq!(json["message"]["name"], "err_msg");
    }

    #[test]
    fn test_statement_serialize_assign() {
        let stmt = Statement::Assign {
            variable: "x".to_string(),
            value: Expr::IntLit { value: 42 },
        };
        let json = serde_json::to_value(&stmt).unwrap();

        assert_eq!(json["type"], "assign");
        assert_eq!(json["variable"], "x");
        assert_eq!(json["value"]["type"], "int_lit");
        assert_eq!(json["value"]["value"], 42);
    }

    #[test]
    fn test_statement_serialize_assign_from_input() {
        let stmt = Statement::Assign {
            variable: "name".to_string(),
            value: Expr::Input,
        };
        let json = serde_json::to_value(&stmt).unwrap();

        assert_eq!(json["type"], "assign");
        assert_eq!(json["variable"], "name");
        assert_eq!(json["value"]["type"], "input");
    }

    #[test]
    fn test_statement_serialize_assign_with_cast() {
        let stmt = Statement::Assign {
            variable: "num".to_string(),
            value: Expr::Cast {
                expr: Box::new(Expr::Input),
                target_type: TypeName::Int,
            },
        };
        let json = serde_json::to_value(&stmt).unwrap();

        assert_eq!(json["type"], "assign");
        assert_eq!(json["variable"], "num");
        assert_eq!(json["value"]["type"], "cast");
        assert_eq!(json["value"]["target_type"], "int");
        assert_eq!(json["value"]["expr"]["type"], "input");
    }
}
