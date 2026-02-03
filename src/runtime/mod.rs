//! Runtime execution engine for Mermaid flowcharts.
//!
//! This module provides the core runtime components for executing programs
//! written in Mermaid flowchart syntax. It handles the complete execution
//! lifecycle from interpreting AST nodes to producing output.
//!
//! # Overview
//!
//! The runtime module is organized into several submodules:
//!
//! - `value`: Runtime value types ([`Value`])
//! - `env`: Variable storage and lookup ([`Environment`])
//! - `eval`: Expression evaluation ([`eval_expr`], [`InputReader`])
//! - `exec`: Statement execution ([`exec_statement`], [`OutputWriter`])
//! - `error`: Runtime error definitions ([`RuntimeError`])
//! - `interpreter`: Main execution loop ([`Interpreter`])
//!
//! # Architecture
//!
//! The execution flow follows this pattern:
//!
//! 1. The [`Interpreter`] is created from a parsed [`Flowchart`](crate::ast::Flowchart)
//! 2. Execution starts from the `Start` node and follows edges
//! 3. [`Process`](crate::ast::Node::Process) nodes execute statements via [`exec_statement`]
//! 4. [`Condition`](crate::ast::Node::Condition) nodes evaluate expressions via [`eval_expr`]
//! 5. Execution terminates when the `End` node is reached
//!
//! # Dependency Injection
//!
//! The runtime uses trait-based dependency injection for I/O operations:
//!
//! - [`InputReader`]: Abstracts input reading (stdin or mock for testing)
//! - [`OutputWriter`]: Abstracts output writing (stdout/stderr or mock for testing)
//!
//! # Example
//!
//! ```ignore
//! use merx::parser;
//! use merx::runtime::Interpreter;
//!
//! let source = r#"
//! flowchart TD
//!     Start --> A[print "Hello, World!"]
//!     A --> End
//! "#;
//!
//! let flowchart = parser::parse(source).unwrap();
//! let mut interpreter = Interpreter::new(flowchart).unwrap();
//! interpreter.run().unwrap();
//! ```

mod env;
mod error;
mod eval;
mod exec;
mod interpreter;
#[cfg(test)]
pub(crate) mod test_helpers;
mod value;

pub use env::Environment;
pub use error::RuntimeError;
pub use eval::{InputReader, StdinReader, eval_expr};
pub use exec::{OutputWriter, StdioWriter, exec_statement};
pub use interpreter::Interpreter;
pub use value::Value;
