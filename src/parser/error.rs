//! Error types for the parser module.
//!
//! This module defines [`ParseError`], the error type returned when parsing
//! fails. It wraps both syntax errors from the pest parser and semantic
//! validation errors detected during AST construction.

use std::fmt;

use pest::error::Error as PestError;

use crate::parser::Rule;

/// An error that occurred during parsing of Mermaid flowchart syntax.
///
/// `ParseError` is returned by [`parse`](super::parse) when the input cannot
/// be successfully parsed into a valid flowchart AST. This includes:
///
/// - **Syntax errors**: Invalid tokens, missing delimiters, malformed expressions
/// - **Semantic errors**: Invalid edge configurations for condition nodes
///
/// # Examples
///
/// ```
/// use merx::parser::parse;
///
/// // Syntax error: missing closing bracket
/// let result = parse("flowchart TD\n    Start --> A[x = 1");
/// assert!(result.is_err());
///
/// // Semantic error: condition without Yes/No edges
/// let result = parse("flowchart TD\n    Start --> A{x > 0?}\n    A --> End");
/// assert!(result.is_err());
/// ```
///
/// # Display
///
/// The error message is human-readable and suitable for display to users.
/// For pest syntax errors, it includes line/column information and context.
#[derive(Debug)]
pub struct ParseError {
    /// The human-readable error message.
    message: String,
}

impl ParseError {
    /// Creates a new `ParseError` with the given message.
    ///
    /// # Arguments
    ///
    /// * `message` - A value that can be converted into a `String` describing the error
    ///
    /// # Examples
    ///
    /// ```
    /// use merx::parser::ParseError;
    ///
    /// let error = ParseError::new("Condition node 'A' is missing 'Yes' edge");
    /// assert_eq!(error.to_string(), "Condition node 'A' is missing 'Yes' edge");
    /// ```
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ParseError {}

/// Converts a pest parsing error into a `ParseError`.
///
/// This allows using the `?` operator to propagate pest errors from
/// `MermaidParser::parse()` calls. The pest error's detailed message
/// (including line/column information) is preserved.
impl From<PestError<Rule>> for ParseError {
    fn from(err: PestError<Rule>) -> Self {
        Self::new(err.to_string())
    }
}
