//! Runtime error types.
//!
//! This module defines [`RuntimeError`], an enum representing all errors
//! that can occur during program execution.
//!
//! # Error Categories
//!
//! Runtime errors fall into several categories:
//!
//! ## Variable Errors
//! - [`UndefinedVariable`](RuntimeError::UndefinedVariable) - Reference to a variable that hasn't been assigned
//!
//! ## Type Errors
//! - [`TypeError`](RuntimeError::TypeError) - Operation applied to incompatible type
//! - [`CastError`](RuntimeError::CastError) - Failed type conversion (e.g., `"abc" as int`)
//!
//! ## Arithmetic Errors
//! - [`DivisionByZero`](RuntimeError::DivisionByZero) - Division or modulo with zero divisor
//!
//! ## Structural Errors
//! - [`MissingStartNode`](RuntimeError::MissingStartNode) - Flowchart lacks a `Start` node
//! - [`MissingEndNode`](RuntimeError::MissingEndNode) - Flowchart lacks an `End` node
//!
//! ## Navigation Errors
//! - [`NoOutgoingEdge`](RuntimeError::NoOutgoingEdge) - Node has no edge to follow
//! - [`NoMatchingConditionEdge`](RuntimeError::NoMatchingConditionEdge) - Condition node lacks required Yes/No edge
//! - [`NodeNotFound`](RuntimeError::NodeNotFound) - Edge references non-existent node
//!
//! ## I/O Errors
//! - [`IoError`](RuntimeError::IoError) - Failed to read input or write output

use std::fmt;

/// An error that occurred during program execution.
///
/// This enum captures all possible runtime failures, from type mismatches
/// to structural problems with the flowchart. All variants implement
/// [`Display`](std::fmt::Display) for user-friendly error messages.
///
/// # Error Handling
///
/// Runtime errors are propagated using Rust's `Result` type. The interpreter
/// stops execution when an error occurs and returns it to the caller.
///
/// # Examples
///
/// ```
/// use merx::runtime::RuntimeError;
///
/// let err = RuntimeError::UndefinedVariable {
///     name: "x".to_string(),
/// };
/// assert_eq!(err.to_string(), "Undefined variable: 'x'");
/// ```
#[derive(Debug, Clone)]
pub enum RuntimeError {
    /// Reference to a variable that has not been assigned.
    ///
    /// This occurs when an expression references a variable name that
    /// has not been previously assigned in the current execution context.
    ///
    /// # Fields
    ///
    /// - `name` - The undefined variable's identifier
    UndefinedVariable { name: String },

    /// Type mismatch in an operation.
    ///
    /// This occurs when an operator or function receives a value of
    /// an unexpected type. For example, arithmetic operations require
    /// integer operands, and logical operations require booleans.
    ///
    /// # Fields
    ///
    /// - `expected` - The type name that was expected
    /// - `actual` - The type name that was actually received
    /// - `operation` - Description of the operation that failed
    TypeError {
        expected: &'static str,
        actual: &'static str,
        operation: String,
    },

    /// Failed type cast.
    ///
    /// This occurs when an explicit type cast cannot be performed.
    /// For example, casting `"abc"` to `int` fails because the string
    /// cannot be parsed as an integer.
    ///
    /// # Fields
    ///
    /// - `from_type` - The source type name
    /// - `to_type` - The target type name
    /// - `value` - String representation of the value that couldn't be cast
    CastError {
        from_type: &'static str,
        to_type: &'static str,
        value: String,
    },

    /// Division or modulo operation with zero divisor.
    ///
    /// Both `/` and `%` operators check for zero divisors and return
    /// this error instead of causing undefined behavior.
    DivisionByZero,

    /// Flowchart is missing a `Start` node.
    ///
    /// Every valid flowchart must have exactly one `Start` node where
    /// execution begins. This is normally caught at parse time, but is
    /// also checked at runtime as a defensive measure for manually
    /// constructed flowcharts.
    MissingStartNode,

    /// Flowchart is missing an `End` node.
    ///
    /// Every valid flowchart must have exactly one `End` node where
    /// execution terminates. This is normally caught at parse time, but is
    /// also checked at runtime as a defensive measure for manually
    /// constructed flowcharts.
    MissingEndNode,

    /// Node has no outgoing edge.
    ///
    /// Non-terminal nodes must have at least one outgoing edge.
    /// Process nodes need one edge, and condition nodes need two
    /// (Yes and No).
    ///
    /// # Fields
    ///
    /// - `node_id` - The identifier of the node lacking an edge
    NoOutgoingEdge { node_id: String },

    /// Condition node lacks an edge for the evaluated result.
    ///
    /// Condition nodes must have both a `Yes` edge and a `No` edge.
    /// This error occurs when the condition evaluates but no matching
    /// edge exists for the result.
    ///
    /// # Fields
    ///
    /// - `node_id` - The identifier of the condition node
    /// - `condition_result` - The boolean result that had no matching edge
    NoMatchingConditionEdge {
        node_id: String,
        condition_result: bool,
    },

    /// Edge references a node that doesn't exist.
    ///
    /// This typically indicates a malformed flowchart where an edge's
    /// destination doesn't match any defined node.
    ///
    /// # Fields
    ///
    /// - `node_id` - The identifier that couldn't be found
    NodeNotFound { node_id: String },

    /// I/O operation failed.
    ///
    /// This wraps errors from reading user input, writing output, or other I/O operations.
    ///
    /// # Fields
    ///
    /// - `message` - Description of the I/O failure
    IoError { message: String },
}

impl fmt::Display for RuntimeError {
    /// Formats a user-friendly error message.
    ///
    /// Error messages are designed to be helpful for debugging:
    /// - They identify the error type
    /// - They include relevant context (variable names, node IDs, types)
    /// - They avoid internal jargon where possible
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeError::UndefinedVariable { name } => {
                write!(f, "Undefined variable: '{}'", name)
            }
            RuntimeError::TypeError {
                expected,
                actual,
                operation,
            } => {
                write!(
                    f,
                    "Type error in {}: expected {}, got {}",
                    operation, expected, actual
                )
            }
            RuntimeError::CastError {
                from_type,
                to_type,
                value,
            } => {
                write!(f, "Cannot cast {} '{}' to {}", from_type, value, to_type)
            }
            RuntimeError::DivisionByZero => {
                write!(f, "Division by zero")
            }
            RuntimeError::MissingStartNode => {
                write!(f, "Missing 'Start' node")
            }
            RuntimeError::MissingEndNode => {
                write!(f, "Missing 'End' node")
            }
            RuntimeError::NoOutgoingEdge { node_id } => {
                write!(f, "No outgoing edge from node '{}'", node_id)
            }
            RuntimeError::NoMatchingConditionEdge {
                node_id,
                condition_result,
            } => {
                write!(
                    f,
                    "No '{}' edge from condition node '{}'",
                    if *condition_result { "Yes" } else { "No" },
                    node_id
                )
            }
            RuntimeError::NodeNotFound { node_id } => {
                write!(f, "Node '{}' not found", node_id)
            }
            RuntimeError::IoError { message } => {
                write!(f, "I/O error: {}", message)
            }
        }
    }
}

impl std::error::Error for RuntimeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_undefined_variable_display() {
        let err = RuntimeError::UndefinedVariable {
            name: "x".to_string(),
        };
        assert_eq!(err.to_string(), "Undefined variable: 'x'");
    }

    #[test]
    fn test_type_error_display() {
        let err = RuntimeError::TypeError {
            expected: "int",
            actual: "str",
            operation: "addition".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Type error in addition: expected int, got str"
        );
    }

    #[test]
    fn test_cast_error_display() {
        let err = RuntimeError::CastError {
            from_type: "str",
            to_type: "int",
            value: "abc".to_string(),
        };
        assert_eq!(err.to_string(), "Cannot cast str 'abc' to int");
    }

    #[test]
    fn test_division_by_zero_display() {
        let err = RuntimeError::DivisionByZero;
        assert_eq!(err.to_string(), "Division by zero");
    }
}
