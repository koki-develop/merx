//! Error types for the parser module.
//!
//! This module defines three error types returned when analysis of Mermaid
//! flowchart source fails:
//!
//! - [`SyntaxError`]: Errors from the PEG parser or AST construction
//! - [`ValidationError`]: Semantic validation errors detected during or after parsing
//! - [`AnalysisError`]: Top-level enum wrapping both, returned by [`parse`](super::parse)

use std::fmt;

use pest::error::Error as PestError;

use crate::parser::Rule;

/// An error that occurred during syntactic parsing of Mermaid flowchart syntax.
///
/// `SyntaxError` is produced when the input contains invalid tokens, missing
/// delimiters, malformed expressions, or other issues detected by the PEG
/// parser or during AST construction.
///
/// # Examples
///
/// ```
/// use merx::parser::SyntaxError;
///
/// let error = SyntaxError::new("integer literal '99999999999999999999' is out of range");
/// assert_eq!(
///     error.to_string(),
///     "integer literal '99999999999999999999' is out of range"
/// );
/// ```
#[derive(Debug)]
pub struct SyntaxError {
    message: String,
}

impl SyntaxError {
    /// Creates a new `SyntaxError` with the given message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for SyntaxError {}

impl From<PestError<Rule>> for SyntaxError {
    fn from(err: PestError<Rule>) -> Self {
        Self::new(err.to_string())
    }
}

/// An error that occurred during semantic validation of a parsed flowchart.
///
/// `ValidationError` is produced when the flowchart structure is syntactically
/// valid but violates semantic rules, such as missing Start/End nodes,
/// duplicate node definitions, or invalid condition node edge configurations.
///
/// # Examples
///
/// ```
/// use merx::parser::ValidationError;
///
/// let error = ValidationError::new("Condition node 'A' is missing 'Yes' edge");
/// assert_eq!(
///     error.to_string(),
///     "Condition node 'A' is missing 'Yes' edge"
/// );
/// ```
#[derive(Debug)]
pub struct ValidationError {
    message: String,
}

impl ValidationError {
    /// Creates a new `ValidationError` with the given message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ValidationError {}

/// An error that occurred during analysis of Mermaid flowchart syntax.
///
/// `AnalysisError` is returned by [`parse`](super::parse) when the input
/// cannot be successfully analyzed into a valid flowchart AST. It wraps both
/// syntax errors and semantic validation errors.
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
/// // Validation error: condition without Yes/No edges
/// let result = parse("flowchart TD\n    Start --> A{x > 0?}\n    A --> End");
/// assert!(result.is_err());
/// ```
#[derive(Debug)]
pub enum AnalysisError {
    /// A syntax error from the PEG parser or AST construction.
    Syntax(SyntaxError),
    /// A semantic validation error.
    Validation(ValidationError),
}

impl fmt::Display for AnalysisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnalysisError::Syntax(e) => write!(f, "Syntax error: {}", e),
            AnalysisError::Validation(e) => write!(f, "Validation error: {}", e),
        }
    }
}

impl std::error::Error for AnalysisError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AnalysisError::Syntax(e) => Some(e),
            AnalysisError::Validation(e) => Some(e),
        }
    }
}

impl From<SyntaxError> for AnalysisError {
    fn from(err: SyntaxError) -> Self {
        AnalysisError::Syntax(err)
    }
}

impl From<ValidationError> for AnalysisError {
    fn from(err: ValidationError) -> Self {
        AnalysisError::Validation(err)
    }
}

impl From<PestError<Rule>> for AnalysisError {
    fn from(err: PestError<Rule>) -> Self {
        AnalysisError::Syntax(SyntaxError::from(err))
    }
}
