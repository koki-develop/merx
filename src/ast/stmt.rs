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
/// | [`Print`](Statement::Print) | `println expr` | Write to stdout |
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
