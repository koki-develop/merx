//! Flowchart root structure and layout direction definitions.
//!
//! This module contains the top-level AST type that represents an entire
//! Mermaid flowchart program.

use serde::Serialize;

use super::{Edge, Node};

/// The root AST node representing a complete Mermaid flowchart program.
///
/// A flowchart consists of a layout direction, a collection of nodes, and
/// edges that connect those nodes. The execution begins at the [`Node::Start`]
/// node and proceeds through connected nodes until reaching [`Node::End`].
///
/// # Structure
///
/// ```text
/// flowchart TD          <- direction
///     Start --> A[...]  <- nodes + edges
///     A --> End
/// ```
///
/// # Invariants
///
/// A valid flowchart must contain:
/// - Exactly one `Start` node (enforced at runtime)
/// - Exactly one `End` node (enforced at runtime)
///
/// Note: Path connectivity from `Start` to `End` is not validated.
/// These invariants are enforced during runtime initialization, not parsing.
///
/// # Serialization
///
/// Serializes to JSON with all fields included:
///
/// ```json
/// {
///   "direction": "TD",
///   "nodes": [...],
///   "edges": [...]
/// }
/// ```
///
/// # See Also
///
/// - [`Node`]: The types of nodes that can appear in a flowchart
/// - [`Edge`]: Connections between nodes
/// - [`crate::parser::parse`]: Creates a `Flowchart` from source text
/// - [`crate::runtime::Interpreter`]: Executes a `Flowchart`
#[derive(Debug, Clone, Serialize)]
pub struct Flowchart {
    /// The layout direction of the flowchart.
    ///
    /// This affects visual rendering but does not impact execution semantics.
    pub direction: Direction,

    /// All nodes defined in the flowchart.
    ///
    /// Includes `Start`, `End`, `Process`, and `Condition` nodes. Each node
    /// (except `Start` and `End`) has a unique identifier used by edges.
    pub nodes: Vec<Node>,

    /// All edges connecting nodes in the flowchart.
    ///
    /// Edges define the control flow between nodes. Condition nodes must have
    /// exactly two outgoing edges labeled `Yes` and `No`.
    pub edges: Vec<Edge>,
}

/// The layout direction for flowchart rendering.
///
/// Specifies how nodes should be arranged visually. This matches the standard
/// Mermaid flowchart direction syntax.
///
/// # Note
///
/// The direction affects only visual layout in Mermaid renderers. The merx
/// interpreter ignores direction for execution purposes—control flow is
/// determined solely by edge connections.
///
/// # Serialization
///
/// Serializes to uppercase strings: `"TD"`, `"TB"`, `"LR"`, `"RL"`, `"BT"`.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Direction {
    /// Top to Down (same as [`Tb`](Direction::Tb)).
    Td,

    /// Top to Bottom.
    Tb,

    /// Left to Right.
    Lr,

    /// Right to Left.
    Rl,

    /// Bottom to Top.
    Bt,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{BinaryOp, EdgeLabel, Expr, Statement};

    #[test]
    fn test_direction_serialize_td() {
        let json = serde_json::to_value(Direction::Td).unwrap();
        assert_eq!(json, "TD");
    }

    #[test]
    fn test_direction_serialize_tb() {
        let json = serde_json::to_value(Direction::Tb).unwrap();
        assert_eq!(json, "TB");
    }

    #[test]
    fn test_direction_serialize_lr() {
        let json = serde_json::to_value(Direction::Lr).unwrap();
        assert_eq!(json, "LR");
    }

    #[test]
    fn test_direction_serialize_rl() {
        let json = serde_json::to_value(Direction::Rl).unwrap();
        assert_eq!(json, "RL");
    }

    #[test]
    fn test_direction_serialize_bt() {
        let json = serde_json::to_value(Direction::Bt).unwrap();
        assert_eq!(json, "BT");
    }

    #[test]
    fn test_direction_serialize_all_variants() {
        let directions = [
            (Direction::Td, "TD"),
            (Direction::Tb, "TB"),
            (Direction::Lr, "LR"),
            (Direction::Rl, "RL"),
            (Direction::Bt, "BT"),
        ];

        for (dir, expected) in directions {
            let json = serde_json::to_value(dir).unwrap();
            assert_eq!(json, expected);
        }
    }

    #[test]
    fn test_flowchart_serialize_minimal() {
        let flowchart = Flowchart {
            direction: Direction::Td,
            nodes: vec![Node::Start, Node::End],
            edges: vec![Edge {
                from: "Start".to_string(),
                to: "End".to_string(),
                label: None,
            }],
        };
        let json = serde_json::to_value(&flowchart).unwrap();

        assert_eq!(json["direction"], "TD");
        assert_eq!(json["nodes"].as_array().unwrap().len(), 2);
        assert_eq!(json["edges"].as_array().unwrap().len(), 1);
        assert_eq!(json["nodes"][0]["type"], "start");
        assert_eq!(json["nodes"][1]["type"], "end");
        assert_eq!(json["edges"][0]["from"], "Start");
        assert_eq!(json["edges"][0]["to"], "End");
        // label is None, so it should be omitted
        assert!(json["edges"][0].get("label").is_none());
    }

    #[test]
    fn test_flowchart_serialize() {
        let flowchart = Flowchart {
            direction: Direction::Lr,
            nodes: vec![
                Node::Start,
                Node::Process {
                    id: "A".to_string(),
                    statements: vec![Statement::Assign {
                        variable: "x".to_string(),
                        value: Expr::IntLit { value: 5 },
                    }],
                },
                Node::Condition {
                    id: "B".to_string(),
                    condition: Expr::Binary {
                        op: BinaryOp::Gt,
                        left: Box::new(Expr::Variable {
                            name: "x".to_string(),
                        }),
                        right: Box::new(Expr::IntLit { value: 0 }),
                    },
                },
                Node::Process {
                    id: "C".to_string(),
                    statements: vec![Statement::Print {
                        expr: Expr::Variable {
                            name: "x".to_string(),
                        },
                    }],
                },
                Node::End,
            ],
            edges: vec![
                Edge {
                    from: "Start".to_string(),
                    to: "A".to_string(),
                    label: None,
                },
                Edge {
                    from: "A".to_string(),
                    to: "B".to_string(),
                    label: None,
                },
                Edge {
                    from: "B".to_string(),
                    to: "C".to_string(),
                    label: Some(EdgeLabel::Yes),
                },
                Edge {
                    from: "B".to_string(),
                    to: "End".to_string(),
                    label: Some(EdgeLabel::No),
                },
                Edge {
                    from: "C".to_string(),
                    to: "End".to_string(),
                    label: None,
                },
            ],
        };
        let json = serde_json::to_value(&flowchart).unwrap();

        // Verify top-level structure
        assert_eq!(json["direction"], "LR");
        assert_eq!(json["nodes"].as_array().unwrap().len(), 5);
        assert_eq!(json["edges"].as_array().unwrap().len(), 5);

        // Verify nodes
        assert_eq!(json["nodes"][0]["type"], "start");
        assert_eq!(json["nodes"][1]["type"], "process");
        assert_eq!(json["nodes"][1]["id"], "A");
        assert_eq!(json["nodes"][2]["type"], "condition");
        assert_eq!(json["nodes"][2]["id"], "B");
        assert_eq!(json["nodes"][3]["type"], "process");
        assert_eq!(json["nodes"][3]["id"], "C");
        assert_eq!(json["nodes"][4]["type"], "end");

        // Verify edges with labels
        assert_eq!(json["edges"][2]["from"], "B");
        assert_eq!(json["edges"][2]["to"], "C");
        assert_eq!(json["edges"][2]["label"], "yes");

        assert_eq!(json["edges"][3]["from"], "B");
        assert_eq!(json["edges"][3]["to"], "End");
        assert_eq!(json["edges"][3]["label"], "no");
    }

    #[test]
    fn test_flowchart_serialize_with_custom_label() {
        let flowchart = Flowchart {
            direction: Direction::Td,
            nodes: vec![Node::Start, Node::End],
            edges: vec![Edge {
                from: "Start".to_string(),
                to: "End".to_string(),
                label: Some(EdgeLabel::Custom("custom label".to_string())),
            }],
        };
        let json = serde_json::to_value(&flowchart).unwrap();

        // Custom labels are serialized untagged (just the string value)
        assert_eq!(json["edges"][0]["label"], "custom label");
    }
}
