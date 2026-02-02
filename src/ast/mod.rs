//! Abstract Syntax Tree (AST) types for Mermaid flowchart programs.
//!
//! This module defines the data structures that represent parsed Mermaid flowchart
//! syntax. The AST is produced by the [`parser`](crate::parser) module and consumed
//! by the [`runtime`](crate::runtime) module for execution.
//!
//! # Module Structure
//!
//! The AST consists of several key components:
//!
//! - [`Flowchart`]: The root node representing an entire flowchart program
//! - [`Node`]: Individual nodes in the flowchart (Start, End, Process, Condition)
//! - [`Edge`]: Connections between nodes with optional labels
//! - [`Statement`]: Executable statements within process nodes
//! - [`Expr`]: Expressions for computations, conditions, and values
//!
//! # Example
//!
//! A simple flowchart program that prints "Hello, World!":
//!
//! ```text
//! flowchart TD
//!     Start --> A[print 'Hello, World!']
//!     A --> End
//! ```
//!
//! This parses into a [`Flowchart`] with:
//! - `direction`: [`Direction::Td`]
//! - `nodes`: `[Start, Process { id: "A", statements: [Print { expr: StrLit }] }, End]`
//! - `edges`: `[Edge { from: "Start", to: "A" }, Edge { from: "A", to: "End" }]`
//!
//! # Serialization
//!
//! All AST types implement [`serde::Serialize`], enabling JSON output for debugging
//! and tooling integration. Enum variants use the `tag = "type"` pattern for clear
//! type discrimination in serialized output.

mod edge;
mod expr;
mod flowchart;
mod node;
mod stmt;

pub use edge::{Edge, EdgeLabel};
pub use expr::{BinaryOp, Expr, TypeName, UnaryOp};
pub use flowchart::{Direction, Flowchart};
pub use node::Node;
pub use stmt::Statement;
