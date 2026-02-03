//! Edge definitions for connecting flowchart nodes.
//!
//! Edges define the control flow between nodes in a Mermaid flowchart. They
//! specify which node to visit next after completing the current node's execution.

/// A directed connection between two nodes in the flowchart.
///
/// Edges define the control flow path through the program. Each edge connects
/// a source node (`from`) to a target node (`to`), optionally with a label
/// for conditional branching.
///
/// # Mermaid Syntax
///
/// ```text
/// A --> B           // Unlabeled edge
/// C -->|Yes| D      // Labeled edge (for conditions)
/// C -->|No| E
/// ```
///
/// # Labels
///
/// Labels are required for edges originating from [`Condition`](super::Node::Condition)
/// nodes. Each condition node must have exactly one `Yes` edge and one `No` edge.
/// Labels are optional (and typically omitted) for edges from other node types.
///
#[derive(Debug, Clone, PartialEq)]
pub struct Edge {
    /// The identifier of the source node.
    ///
    /// Must match the `id` of an existing node in the flowchart.
    pub from: String,

    /// The identifier of the target node.
    ///
    /// Must match the `id` of an existing node in the flowchart.
    pub to: String,

    /// An optional label for conditional branching.
    ///
    /// Required for edges from condition nodes; should be [`EdgeLabel::Yes`]
    /// or [`EdgeLabel::No`]. Custom labels are allowed but have no special
    /// meaning to the interpreter.
    pub label: Option<EdgeLabel>,
}

/// A label attached to an edge for conditional branching.
///
/// Edge labels determine which path to follow when leaving a condition node.
/// The interpreter recognizes `Yes` and `No` as special values for boolean
/// branching.
///
/// # Mermaid Syntax
///
/// ```text
/// A -->|Yes| B      // Yes label
/// A -->|No| C       // No label
/// A -->|custom| D   // Custom label (not used for branching)
/// ```
///
#[derive(Debug, Clone, PartialEq)]
pub enum EdgeLabel {
    /// The "Yes" branch for true conditions.
    ///
    /// This edge is followed when a condition expression evaluates to `true`.
    Yes,

    /// The "No" branch for false conditions.
    ///
    /// This edge is followed when a condition expression evaluates to `false`.
    No,

    /// A custom label string.
    ///
    /// Custom labels are parsed but have no special meaning to the interpreter.
    /// They may be used for documentation or visual purposes in the Mermaid
    /// diagram.
    Custom(String),
}

impl EdgeLabel {
    /// Checks whether this label is a conditional branch label (`Yes` or `No`).
    ///
    /// This is used during parsing to validate that condition nodes have the
    /// required branch edges.
    ///
    /// # Returns
    ///
    /// `true` if the label is [`Yes`](EdgeLabel::Yes) or [`No`](EdgeLabel::No),
    /// `false` for [`Custom`](EdgeLabel::Custom) labels.
    ///
    /// # Examples
    ///
    /// ```
    /// use merx::ast::EdgeLabel;
    ///
    /// assert!(EdgeLabel::Yes.is_yes_or_no());
    /// assert!(EdgeLabel::No.is_yes_or_no());
    /// assert!(!EdgeLabel::Custom("maybe".to_string()).is_yes_or_no());
    /// ```
    pub fn is_yes_or_no(&self) -> bool {
        matches!(self, EdgeLabel::Yes | EdgeLabel::No)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_label_is_yes_or_no_yes() {
        assert!(EdgeLabel::Yes.is_yes_or_no());
    }

    #[test]
    fn test_edge_label_is_yes_or_no_no() {
        assert!(EdgeLabel::No.is_yes_or_no());
    }

    #[test]
    fn test_edge_label_is_yes_or_no_custom() {
        assert!(!EdgeLabel::Custom("maybe".to_string()).is_yes_or_no());
        assert!(!EdgeLabel::Custom("".to_string()).is_yes_or_no());
        assert!(!EdgeLabel::Custom("YES".to_string()).is_yes_or_no());
        assert!(!EdgeLabel::Custom("yes".to_string()).is_yes_or_no());
    }

    #[test]
    fn test_edge_label_is_yes_or_no_none() {
        let label: Option<EdgeLabel> = None;
        assert!(label.is_none());

        // When label is None, is_yes_or_no cannot be called directly.
        // This test verifies the pattern used in the codebase.
        let is_yes_or_no = label.as_ref().is_some_and(|l| l.is_yes_or_no());
        assert!(!is_yes_or_no);
    }
}
