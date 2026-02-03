//! Flowchart root structure and layout direction definitions.
//!
//! This module contains the top-level AST type that represents an entire
//! Mermaid flowchart program.

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
/// - Exactly one `Start` node (enforced at parse time and runtime)
/// - Exactly one `End` node (enforced at parse time and runtime)
///
/// Note: Path connectivity from `Start` to `End` is not validated.
///
/// # See Also
///
/// - [`Node`]: The types of nodes that can appear in a flowchart
/// - [`Edge`]: Connections between nodes
/// - [`crate::parser::parse`]: Creates a `Flowchart` from source text
/// - [`crate::runtime::Interpreter`]: Executes a `Flowchart`
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone, Copy, PartialEq)]
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
