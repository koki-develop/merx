//! Flowchart node definitions.
//!
//! Nodes are the fundamental building blocks of a Mermaid flowchart. Each node
//! represents a point in the program's control flow.

use serde::Serialize;

use super::{Expr, Statement};

/// A node in the flowchart representing a point in program execution.
///
/// Flowcharts consist of nodes connected by edges. Execution flows from node
/// to node following edge connections.
///
/// # Node Types
///
/// | Type | Mermaid Syntax | Purpose |
/// |------|----------------|---------|
/// | [`Start`](Node::Start) | `Start` | Entry point (exactly one required) |
/// | [`End`](Node::End) | `End` | Exit point (at least one required) |
/// | [`Process`](Node::Process) | `id[statements]` | Execute statements |
/// | [`Condition`](Node::Condition) | `id{expr?}` | Branch based on condition |
///
/// # Examples
///
/// ```text
/// flowchart TD
///     Start --> A[x = 5]           // Process node
///     A --> B{x > 0?}              // Condition node
///     B -->|Yes| C[print x]
///     B -->|No| End
///     C --> End                    // End node
/// ```
///
/// # Serialization
///
/// Uses tagged enum serialization with `"type"` discriminator:
///
/// ```json
/// { "type": "process", "id": "A", "statements": [...] }
/// { "type": "condition", "id": "B", "condition": {...} }
/// { "type": "start" }
/// { "type": "end" }
/// ```
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Node {
    /// The entry point of the flowchart.
    ///
    /// Every flowchart must have exactly one `Start` node. Execution begins
    /// here and follows outgoing edges. The `Start` node has a fixed identifier
    /// `"Start"` and cannot contain statements.
    ///
    /// # Mermaid Syntax
    ///
    /// ```text
    /// Start --> NextNode
    /// Start([Begin]) --> NextNode
    /// ```
    Start {
        /// Optional display label for documentation purposes.
        /// Does not affect execution.
        #[serde(skip_serializing_if = "Option::is_none")]
        label: Option<String>,
    },

    /// An exit point of the flowchart.
    ///
    /// Execution terminates when reaching an `End` node. A flowchart must have
    /// exactly one `End` node. The `End` node has a fixed identifier `"End"`.
    ///
    /// # Mermaid Syntax
    ///
    /// ```text
    /// PreviousNode --> End
    /// PreviousNode --> End([Finish])
    /// ```
    End {
        /// Optional display label for documentation purposes.
        /// Does not affect execution.
        #[serde(skip_serializing_if = "Option::is_none")]
        label: Option<String>,
    },

    /// A processing node that executes a sequence of statements.
    ///
    /// Process nodes contain one or more statements separated by semicolons.
    /// After executing all statements, control flows to the single outgoing edge.
    ///
    /// # Mermaid Syntax
    ///
    /// ```text
    /// A[x = 1; print x]
    /// ```
    ///
    /// # Fields
    ///
    /// - `id`: Unique identifier for referencing in edges
    /// - `statements`: Sequence of statements to execute
    Process {
        /// The unique identifier for this node.
        ///
        /// Used by [`Edge`](super::Edge) to reference this node as a source or target.
        id: String,

        /// The statements to execute when this node is reached.
        ///
        /// Statements execute sequentially. See [`Statement`] for available
        /// statement types.
        statements: Vec<Statement>,
    },

    /// A conditional branching node.
    ///
    /// Evaluates a boolean expression and follows one of two outgoing edges
    /// based on the result. Must have exactly two outgoing edges labeled
    /// [`Yes`](super::EdgeLabel::Yes) and [`No`](super::EdgeLabel::No).
    ///
    /// # Mermaid Syntax
    ///
    /// ```text
    /// B{x > 0?}
    /// B -->|Yes| TrueNode
    /// B -->|No| FalseNode
    /// ```
    ///
    /// # Fields
    ///
    /// - `id`: Unique identifier for referencing in edges
    /// - `condition`: Expression that must evaluate to a boolean
    Condition {
        /// The unique identifier for this node.
        ///
        /// Used by [`Edge`](super::Edge) to reference this node as a source or target.
        id: String,

        /// The expression to evaluate for branching.
        ///
        /// Must evaluate to a boolean value at runtime. If the result is `true`,
        /// execution follows the `Yes` edge; otherwise, the `No` edge.
        condition: Expr,
    },
}

impl Node {
    /// Returns the identifier of this node.
    ///
    /// For `Start` and `End` nodes, returns the fixed strings `"Start"` and
    /// `"End"` respectively. For `Process` and `Condition` nodes, returns
    /// the user-defined identifier.
    ///
    /// # Examples
    ///
    /// ```
    /// use merx::ast::Node;
    ///
    /// assert_eq!(Node::Start { label: None }.id(), "Start");
    /// assert_eq!(Node::End { label: None }.id(), "End");
    /// ```
    pub fn id(&self) -> &str {
        match self {
            Node::Start { .. } => "Start",
            Node::End { .. } => "End",
            Node::Process { id, .. } => id,
            Node::Condition { id, .. } => id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{BinaryOp, Expr};

    #[test]
    fn test_node_serialize_start() {
        let node = Node::Start { label: None };
        let json = serde_json::to_value(&node).unwrap();

        assert_eq!(json["type"], "start");
        // Start node without label has no other fields
        assert!(json.get("id").is_none());
        assert!(json.get("label").is_none());
    }

    #[test]
    fn test_node_serialize_start_with_label() {
        let node = Node::Start {
            label: Some("Begin".to_string()),
        };
        let json = serde_json::to_value(&node).unwrap();

        assert_eq!(json["type"], "start");
        assert_eq!(json["label"], "Begin");
    }

    #[test]
    fn test_node_serialize_end() {
        let node = Node::End { label: None };
        let json = serde_json::to_value(&node).unwrap();

        assert_eq!(json["type"], "end");
        // End node without label has no other fields
        assert!(json.get("id").is_none());
        assert!(json.get("label").is_none());
    }

    #[test]
    fn test_node_serialize_end_with_label() {
        let node = Node::End {
            label: Some("Finish".to_string()),
        };
        let json = serde_json::to_value(&node).unwrap();

        assert_eq!(json["type"], "end");
        assert_eq!(json["label"], "Finish");
    }

    #[test]
    fn test_node_serialize_process() {
        let node = Node::Process {
            id: "A".to_string(),
            statements: vec![Statement::Println {
                expr: Expr::StrLit {
                    value: "hello".to_string(),
                },
            }],
        };
        let json = serde_json::to_value(&node).unwrap();

        assert_eq!(json["type"], "process");
        assert_eq!(json["id"], "A");
        assert!(json["statements"].is_array());
        assert_eq!(json["statements"].as_array().unwrap().len(), 1);
        assert_eq!(json["statements"][0]["type"], "println");
    }

    #[test]
    fn test_node_serialize_process_multiple_statements() {
        let node = Node::Process {
            id: "B".to_string(),
            statements: vec![
                Statement::Assign {
                    variable: "x".to_string(),
                    value: Expr::IntLit { value: 5 },
                },
                Statement::Println {
                    expr: Expr::Variable {
                        name: "x".to_string(),
                    },
                },
            ],
        };
        let json = serde_json::to_value(&node).unwrap();

        assert_eq!(json["type"], "process");
        assert_eq!(json["id"], "B");
        assert_eq!(json["statements"].as_array().unwrap().len(), 2);
        assert_eq!(json["statements"][0]["type"], "assign");
        assert_eq!(json["statements"][1]["type"], "println");
    }

    #[test]
    fn test_node_serialize_process_empty_statements() {
        let node = Node::Process {
            id: "Empty".to_string(),
            statements: vec![],
        };
        let json = serde_json::to_value(&node).unwrap();

        assert_eq!(json["type"], "process");
        assert_eq!(json["id"], "Empty");
        assert!(json["statements"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_node_serialize_condition() {
        let node = Node::Condition {
            id: "C".to_string(),
            condition: Expr::Binary {
                op: BinaryOp::Gt,
                left: Box::new(Expr::Variable {
                    name: "x".to_string(),
                }),
                right: Box::new(Expr::IntLit { value: 0 }),
            },
        };
        let json = serde_json::to_value(&node).unwrap();

        assert_eq!(json["type"], "condition");
        assert_eq!(json["id"], "C");
        assert_eq!(json["condition"]["type"], "binary");
        assert_eq!(json["condition"]["op"], "gt");
    }

    #[test]
    fn test_node_serialize_condition_simple() {
        let node = Node::Condition {
            id: "D".to_string(),
            condition: Expr::BoolLit { value: true },
        };
        let json = serde_json::to_value(&node).unwrap();

        assert_eq!(json["type"], "condition");
        assert_eq!(json["id"], "D");
        assert_eq!(json["condition"]["type"], "bool_lit");
        assert_eq!(json["condition"]["value"], true);
    }
}
