//! Flowchart node definitions.
//!
//! Nodes are the fundamental building blocks of a Mermaid flowchart. Each node
//! represents a point in the program's control flow.

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
#[derive(Debug, Clone, PartialEq)]
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
