use std::fmt;

/// Runtime error variants.
#[derive(Debug, Clone)]
pub enum RuntimeError {
    /// Reference to an undefined variable.
    UndefinedVariable { name: String },

    /// Type mismatch error.
    TypeError {
        expected: &'static str,
        actual: &'static str,
        operation: String,
    },

    /// Type cast failure.
    CastError {
        from_type: &'static str,
        to_type: &'static str,
        value: String,
    },

    /// Division by zero.
    DivisionByZero,

    /// Start node not found.
    MissingStartNode,

    /// End node not found.
    MissingEndNode,

    /// Multiple Start nodes found.
    MultipleStartNodes,

    /// Multiple End nodes found.
    MultipleEndNodes,

    /// No outgoing edge from a node.
    NoOutgoingEdge { node_id: String },

    /// No matching edge for condition result.
    NoMatchingConditionEdge {
        node_id: String,
        condition_result: bool,
    },

    /// Node not found.
    NodeNotFound { node_id: String },

    /// I/O error.
    IoError { message: String },
}

impl fmt::Display for RuntimeError {
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
            RuntimeError::MultipleStartNodes => {
                write!(f, "Multiple 'Start' nodes found")
            }
            RuntimeError::MultipleEndNodes => {
                write!(f, "Multiple 'End' nodes found")
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
