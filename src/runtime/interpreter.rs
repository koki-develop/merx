//! Flowchart interpreter and execution engine.
//!
//! This module provides the [`Interpreter`] struct, which executes a parsed
//! Mermaid flowchart as a program.
//!
//! # Execution Model
//!
//! The interpreter follows a graph-based execution model:
//!
//! 1. **Initialization**: Build node and edge lookup tables from the flowchart
//! 2. **Validation**: Ensure exactly one Start and one End node exist
//! 3. **Execution**: Starting from `Start`, follow edges through the graph
//! 4. **Termination**: Execution ends when the `End` node is reached
//!
//! # Node Handling
//!
//! | Node Type | Behavior |
//! |-----------|----------|
//! | `Start` | Entry point; follow single outgoing edge |
//! | `End` | Terminal; execution stops |
//! | `Process` | Execute all statements; follow single outgoing edge |
//! | `Condition` | Evaluate expression; follow Yes or No edge based on result |
//!
//! # Control Flow
//!
//! - **Sequential**: Process nodes have one outgoing edge
//! - **Conditional**: Condition nodes have two labeled edges (Yes/No)
//! - **Loops**: Edges can point to earlier nodes, creating loops
//!
//! # I/O Abstraction
//!
//! The interpreter is generic over input and output types, allowing:
//! - Production use with stdin/stdout
//! - Testing with mock I/O
//!
//! # Example
//!
//! ```ignore
//! use merx::parser;
//! use merx::runtime::Interpreter;
//!
//! let source = r#"
//! flowchart TD
//!     Start --> A[x = input]
//!     A --> B{x == "quit"}
//!     B -->|Yes| End
//!     B -->|No| C[print x]
//!     C --> A
//! "#;
//!
//! let flowchart = parser::parse(source).unwrap();
//! let mut interpreter = Interpreter::new(flowchart).unwrap();
//! interpreter.run().unwrap();
//! ```

use std::collections::HashMap;
use std::io;

use crate::ast::{Edge, EdgeLabel, Flowchart, Node};

use super::env::Environment;
use super::error::RuntimeError;
use super::eval::{InputReader, StdinReader, eval_expr};
use super::exec::{OutputWriter, StdioWriter, exec_statement};

/// The main execution engine for Mermaid flowchart programs.
///
/// The interpreter manages the execution state including:
/// - Node and edge lookup tables
/// - Current execution position
/// - Variable environment
/// - I/O handlers
///
/// # Type Parameters
///
/// - `R` - Input reader type, implements [`InputReader`]
/// - `W` - Output writer type, implements [`OutputWriter`]
///
/// # Construction
///
/// Use [`Interpreter::new`] for stdin/stdout, or [`Interpreter::with_io`]
/// for custom I/O handlers.
///
/// # Execution
///
/// Call [`run`](Interpreter::run) to execute the program. Execution is
/// synchronous and completes when reaching the `End` node or encountering
/// an error.
///
/// # Examples
///
/// ```ignore
/// use merx::parser;
/// use merx::runtime::Interpreter;
///
/// let flowchart = parser::parse(source).unwrap();
///
/// // With default I/O (stdin/stdout)
/// let mut interpreter = Interpreter::new(flowchart).unwrap();
/// interpreter.run().unwrap();
/// ```
pub struct Interpreter<R: InputReader, W: OutputWriter> {
    /// Lookup table mapping node IDs to their definitions.
    ///
    /// Built during construction from the flowchart's node list.
    nodes: HashMap<String, Node>,

    /// Lookup table mapping node IDs to their outgoing edges.
    ///
    /// Process nodes typically have one edge; condition nodes have two
    /// (labeled Yes and No).
    outgoing_edges: HashMap<String, Vec<Edge>>,

    /// The ID of the node currently being executed.
    ///
    /// Starts at "Start" and updated as edges are followed.
    current_node_id: String,

    /// The variable environment storing all variable bindings.
    env: Environment,

    /// The input source for `input` expressions.
    input_reader: R,

    /// The output destination for print/error statements.
    output_writer: W,
}

impl Interpreter<StdinReader<io::BufReader<io::Stdin>>, StdioWriter> {
    /// Creates an interpreter with default I/O (stdin/stdout).
    ///
    /// This is the standard way to create an interpreter for command-line
    /// execution.
    ///
    /// # Arguments
    ///
    /// * `flowchart` - The parsed flowchart to execute
    ///
    /// # Returns
    ///
    /// An interpreter ready to run, or an error if the flowchart is invalid.
    ///
    /// # Errors
    ///
    /// - [`RuntimeError::MissingStartNode`] - No `Start` node found
    /// - [`RuntimeError::MissingEndNode`] - No `End` node found
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use merx::parser;
    /// use merx::runtime::Interpreter;
    ///
    /// let flowchart = parser::parse(source).unwrap();
    /// let mut interpreter = Interpreter::new(flowchart).unwrap();
    /// ```
    pub fn new(flowchart: Flowchart) -> Result<Self, RuntimeError> {
        Self::with_io(flowchart, StdinReader::new(), StdioWriter::new())
    }
}

impl<R: InputReader, W: OutputWriter> Interpreter<R, W> {
    /// Creates an interpreter with custom I/O handlers.
    ///
    /// Use this for testing or when you need to capture/provide I/O
    /// programmatically.
    ///
    /// # Arguments
    ///
    /// * `flowchart` - The parsed flowchart to execute
    /// * `input_reader` - Custom input source
    /// * `output_writer` - Custom output destination
    ///
    /// # Returns
    ///
    /// An interpreter ready to run, or an error if the flowchart is invalid.
    ///
    /// # Errors
    ///
    /// Same as [`Interpreter::new`]:
    /// - [`RuntimeError::MissingStartNode`]
    /// - [`RuntimeError::MissingEndNode`]
    ///
    /// # Implementation Details
    ///
    /// Construction performs these steps:
    /// 1. Build node lookup table from flowchart nodes
    /// 2. Count Start and End nodes for validation
    /// 3. Build edge lookup table from flowchart edges
    /// 4. Initialize empty environment
    pub fn with_io(
        flowchart: Flowchart,
        input_reader: R,
        output_writer: W,
    ) -> Result<Self, RuntimeError> {
        // Build the node map
        let mut nodes: HashMap<String, Node> = HashMap::new();
        let mut start_count = 0;
        let mut end_count = 0;

        for node in flowchart.nodes {
            let id = node.id().to_string();
            match &node {
                Node::Start { .. } => start_count += 1,
                Node::End { .. } => end_count += 1,
                _ => {}
            }
            nodes.insert(id, node);
        }

        // Validate Start/End nodes
        if start_count == 0 {
            return Err(RuntimeError::MissingStartNode);
        }
        if end_count == 0 {
            return Err(RuntimeError::MissingEndNode);
        }

        // Build the outgoing edge map
        let mut outgoing_edges: HashMap<String, Vec<Edge>> = HashMap::new();
        for edge in flowchart.edges {
            outgoing_edges
                .entry(edge.from.clone())
                .or_default()
                .push(edge);
        }

        Ok(Self {
            nodes,
            outgoing_edges,
            current_node_id: "Start".to_string(),
            env: Environment::new(),
            input_reader,
            output_writer,
        })
    }

    /// Executes the program from start to completion.
    ///
    /// This is the main execution loop. It processes nodes sequentially,
    /// following edges until the `End` node is reached or an error occurs.
    ///
    /// # Execution Flow
    ///
    /// For each node:
    /// 1. Look up the current node by ID
    /// 2. Execute based on node type:
    ///    - `Start`: Follow the outgoing edge
    ///    - `End`: Return successfully
    ///    - `Process`: Execute all statements, then follow edge
    ///    - `Condition`: Evaluate condition, follow Yes/No edge
    /// 3. Repeat until End or error
    ///
    /// # Returns
    ///
    /// `Ok(())` when execution reaches the `End` node normally.
    ///
    /// # Errors
    ///
    /// Any runtime error stops execution immediately:
    ///
    /// - Statement execution errors (type mismatch, undefined variable, etc.)
    /// - Navigation errors (missing edge, missing node)
    /// - I/O errors (input reading failed)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut interpreter = Interpreter::new(flowchart).unwrap();
    ///
    /// match interpreter.run() {
    ///     Ok(()) => println!("Program completed successfully"),
    ///     Err(e) => eprintln!("Runtime error: {}", e),
    /// }
    /// ```
    pub fn run(&mut self) -> Result<(), RuntimeError> {
        loop {
            let node = self
                .nodes
                .get(&self.current_node_id)
                .ok_or_else(|| RuntimeError::NodeNotFound {
                    node_id: self.current_node_id.clone(),
                })?
                .clone();

            match &node {
                Node::Start { .. } => {
                    // Move to the next node from Start
                    self.move_to_next()?;
                }
                Node::End { .. } => {
                    // Terminate
                    return Ok(());
                }
                Node::Process { statements, .. } => {
                    // Execute all statements
                    for stmt in statements {
                        exec_statement(
                            stmt,
                            &mut self.env,
                            &mut self.input_reader,
                            &mut self.output_writer,
                        )?;
                    }
                    self.move_to_next()?;
                }
                Node::Condition { condition, .. } => {
                    // Evaluate the condition
                    let val = eval_expr(condition, &self.env, &mut self.input_reader)?;
                    let result = val.as_bool().ok_or_else(|| RuntimeError::TypeError {
                        expected: "bool",
                        actual: val.type_name(),
                        operation: "condition evaluation".to_string(),
                    })?;
                    self.move_to_condition_branch(result)?;
                }
            }
        }
    }

    /// Follows an unconditional edge to the next node.
    ///
    /// Used by `Start` and `Process` nodes which have exactly one
    /// outgoing edge (not labeled with Yes/No).
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError::NoOutgoingEdge`] if the current node
    /// has no outgoing edges.
    fn move_to_next(&mut self) -> Result<(), RuntimeError> {
        let edges = self
            .outgoing_edges
            .get(&self.current_node_id)
            .ok_or_else(|| RuntimeError::NoOutgoingEdge {
                node_id: self.current_node_id.clone(),
            })?;

        if edges.is_empty() {
            return Err(RuntimeError::NoOutgoingEdge {
                node_id: self.current_node_id.clone(),
            });
        }

        // Use the first edge from normal nodes
        self.current_node_id = edges[0].to.clone();
        Ok(())
    }

    /// Follows a conditional edge based on the condition result.
    ///
    /// Used by `Condition` nodes which have two outgoing edges:
    /// one labeled `Yes` and one labeled `No`.
    ///
    /// # Arguments
    ///
    /// * `condition_result` - The boolean result of evaluating the condition
    ///   - `true` follows the `Yes` edge
    ///   - `false` follows the `No` edge
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError::NoMatchingConditionEdge`] if no edge with
    /// the required label exists.
    fn move_to_condition_branch(&mut self, condition_result: bool) -> Result<(), RuntimeError> {
        let edges = self
            .outgoing_edges
            .get(&self.current_node_id)
            .ok_or_else(|| RuntimeError::NoOutgoingEdge {
                node_id: self.current_node_id.clone(),
            })?;

        let target_label = if condition_result {
            EdgeLabel::Yes
        } else {
            EdgeLabel::No
        };

        for edge in edges {
            if let Some(label) = &edge.label
                && matches!(
                    (label, &target_label),
                    (EdgeLabel::Yes, EdgeLabel::Yes) | (EdgeLabel::No, EdgeLabel::No)
                )
            {
                self.current_node_id = edge.to.clone();
                return Ok(());
            }
        }

        Err(RuntimeError::NoMatchingConditionEdge {
            node_id: self.current_node_id.clone(),
            condition_result,
        })
    }

    /// Consumes the interpreter and returns the output writer.
    ///
    /// This is useful for testing when you need to inspect the output
    /// produced during execution.
    ///
    /// # Returns
    ///
    /// The output writer that was used during execution.
    pub fn into_output_writer(self) -> W {
        self.output_writer
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Direction, Expr, Statement};

    /// Mock input reader for testing.
    struct MockInputReader {
        lines: Vec<String>,
        index: usize,
    }

    impl MockInputReader {
        fn new(lines: Vec<&str>) -> Self {
            Self {
                lines: lines.into_iter().map(|s| s.to_string()).collect(),
                index: 0,
            }
        }
    }

    impl InputReader for MockInputReader {
        fn read_line(&mut self) -> Result<String, RuntimeError> {
            if self.index < self.lines.len() {
                let line = self.lines[self.index].clone();
                self.index += 1;
                Ok(line)
            } else {
                Err(RuntimeError::IoError {
                    message: "No more input".to_string(),
                })
            }
        }
    }

    /// Mock output writer for testing.
    struct MockOutputWriter {
        pub stdout: Vec<String>,
        pub stderr: Vec<String>,
    }

    impl MockOutputWriter {
        fn new() -> Self {
            Self {
                stdout: Vec::new(),
                stderr: Vec::new(),
            }
        }
    }

    impl OutputWriter for MockOutputWriter {
        fn write_stdout(&mut self, s: &str) {
            self.stdout.push(s.to_string());
        }

        fn write_stdout_no_newline(&mut self, s: &str) {
            self.stdout.push(s.to_string());
        }

        fn write_stderr(&mut self, s: &str) {
            self.stderr.push(s.to_string());
        }
    }

    fn create_simple_flowchart() -> Flowchart {
        // Start --> A[print 'hello'] --> End
        Flowchart {
            direction: Direction::Td,
            nodes: vec![
                Node::Start { label: None },
                Node::Process {
                    id: "A".to_string(),
                    statements: vec![Statement::Println {
                        expr: Expr::StrLit {
                            value: "hello".to_string(),
                        },
                    }],
                },
                Node::End { label: None },
            ],
            edges: vec![
                Edge {
                    from: "Start".to_string(),
                    to: "A".to_string(),
                    label: None,
                },
                Edge {
                    from: "A".to_string(),
                    to: "End".to_string(),
                    label: None,
                },
            ],
        }
    }

    fn create_condition_flowchart() -> Flowchart {
        // Start --> A[x = 5] --> B{x > 3?}
        // B -->|Yes| C[print 'big'] --> End
        // B -->|No| D[print 'small'] --> End
        Flowchart {
            direction: Direction::Td,
            nodes: vec![
                Node::Start { label: None },
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
                        op: crate::ast::BinaryOp::Gt,
                        left: Box::new(Expr::Variable {
                            name: "x".to_string(),
                        }),
                        right: Box::new(Expr::IntLit { value: 3 }),
                    },
                },
                Node::Process {
                    id: "C".to_string(),
                    statements: vec![Statement::Println {
                        expr: Expr::StrLit {
                            value: "big".to_string(),
                        },
                    }],
                },
                Node::Process {
                    id: "D".to_string(),
                    statements: vec![Statement::Println {
                        expr: Expr::StrLit {
                            value: "small".to_string(),
                        },
                    }],
                },
                Node::End { label: None },
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
                    to: "D".to_string(),
                    label: Some(EdgeLabel::No),
                },
                Edge {
                    from: "C".to_string(),
                    to: "End".to_string(),
                    label: None,
                },
                Edge {
                    from: "D".to_string(),
                    to: "End".to_string(),
                    label: None,
                },
            ],
        }
    }

    #[test]
    fn test_simple_execution() {
        let flowchart = create_simple_flowchart();
        let input = MockInputReader::new(vec![]);
        let output = MockOutputWriter::new();

        let mut interpreter = Interpreter::with_io(flowchart, input, output).unwrap();
        interpreter.run().unwrap();

        assert_eq!(interpreter.output_writer.stdout, vec!["hello"]);
    }

    #[test]
    fn test_condition_yes_branch() {
        let flowchart = create_condition_flowchart();
        let input = MockInputReader::new(vec![]);
        let output = MockOutputWriter::new();

        let mut interpreter = Interpreter::with_io(flowchart, input, output).unwrap();
        interpreter.run().unwrap();

        assert_eq!(interpreter.output_writer.stdout, vec!["big"]);
    }

    #[test]
    fn test_missing_start_node() {
        let flowchart = Flowchart {
            direction: Direction::Td,
            nodes: vec![Node::End { label: None }],
            edges: vec![],
        };
        let input = MockInputReader::new(vec![]);
        let output = MockOutputWriter::new();

        let result = Interpreter::with_io(flowchart, input, output);
        assert!(matches!(result, Err(RuntimeError::MissingStartNode)));
    }

    #[test]
    fn test_missing_end_node() {
        let flowchart = Flowchart {
            direction: Direction::Td,
            nodes: vec![Node::Start { label: None }],
            edges: vec![],
        };
        let input = MockInputReader::new(vec![]);
        let output = MockOutputWriter::new();

        let result = Interpreter::with_io(flowchart, input, output);
        assert!(matches!(result, Err(RuntimeError::MissingEndNode)));
    }

    #[test]
    fn test_no_outgoing_edge() {
        let flowchart = Flowchart {
            direction: Direction::Td,
            nodes: vec![Node::Start { label: None }, Node::End { label: None }],
            edges: vec![], // No edge from Start
        };
        let input = MockInputReader::new(vec![]);
        let output = MockOutputWriter::new();

        let mut interpreter = Interpreter::with_io(flowchart, input, output).unwrap();
        let result = interpreter.run();
        assert!(matches!(result, Err(RuntimeError::NoOutgoingEdge { .. })));
    }

    #[test]
    fn test_error_statement() {
        // Start --> A[error 'test error'] --> End
        let flowchart = Flowchart {
            direction: Direction::Td,
            nodes: vec![
                Node::Start { label: None },
                Node::Process {
                    id: "A".to_string(),
                    statements: vec![Statement::Error {
                        message: Expr::StrLit {
                            value: "test error".to_string(),
                        },
                    }],
                },
                Node::End { label: None },
            ],
            edges: vec![
                Edge {
                    from: "Start".to_string(),
                    to: "A".to_string(),
                    label: None,
                },
                Edge {
                    from: "A".to_string(),
                    to: "End".to_string(),
                    label: None,
                },
            ],
        };
        let input = MockInputReader::new(vec![]);
        let output = MockOutputWriter::new();

        let mut interpreter = Interpreter::with_io(flowchart, input, output).unwrap();
        let result = interpreter.run();

        assert!(result.is_ok());
        assert_eq!(interpreter.output_writer.stderr, vec!["test error"]);
    }

    #[test]
    fn test_loop_execution() {
        // Start --> A[n = 1] --> B{n <= 3?}
        // B -->|Yes| C[print n; n = n + 1] --> B
        // B -->|No| End
        let flowchart = Flowchart {
            direction: Direction::Td,
            nodes: vec![
                Node::Start { label: None },
                Node::Process {
                    id: "A".to_string(),
                    statements: vec![Statement::Assign {
                        variable: "n".to_string(),
                        value: Expr::IntLit { value: 1 },
                    }],
                },
                Node::Condition {
                    id: "B".to_string(),
                    condition: Expr::Binary {
                        op: crate::ast::BinaryOp::Le,
                        left: Box::new(Expr::Variable {
                            name: "n".to_string(),
                        }),
                        right: Box::new(Expr::IntLit { value: 3 }),
                    },
                },
                Node::Process {
                    id: "C".to_string(),
                    statements: vec![
                        Statement::Println {
                            expr: Expr::Variable {
                                name: "n".to_string(),
                            },
                        },
                        Statement::Assign {
                            variable: "n".to_string(),
                            value: Expr::Binary {
                                op: crate::ast::BinaryOp::Add,
                                left: Box::new(Expr::Variable {
                                    name: "n".to_string(),
                                }),
                                right: Box::new(Expr::IntLit { value: 1 }),
                            },
                        },
                    ],
                },
                Node::End { label: None },
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
                    to: "B".to_string(),
                    label: None,
                },
            ],
        };
        let input = MockInputReader::new(vec![]);
        let output = MockOutputWriter::new();

        let mut interpreter = Interpreter::with_io(flowchart, input, output).unwrap();
        interpreter.run().unwrap();

        assert_eq!(interpreter.output_writer.stdout, vec!["1", "2", "3"]);
    }

    #[test]
    fn test_condition_non_bool_result() {
        // Start --> A{42} --> End (condition evaluates to int, not bool)
        let flowchart = Flowchart {
            direction: Direction::Td,
            nodes: vec![
                Node::Start { label: None },
                Node::Condition {
                    id: "A".to_string(),
                    condition: Expr::IntLit { value: 42 },
                },
                Node::End { label: None },
            ],
            edges: vec![
                Edge {
                    from: "Start".to_string(),
                    to: "A".to_string(),
                    label: None,
                },
                Edge {
                    from: "A".to_string(),
                    to: "End".to_string(),
                    label: Some(EdgeLabel::Yes),
                },
            ],
        };
        let input = MockInputReader::new(vec![]);
        let output = MockOutputWriter::new();

        let mut interpreter = Interpreter::with_io(flowchart, input, output).unwrap();
        let result = interpreter.run();

        assert!(matches!(
            result,
            Err(RuntimeError::TypeError {
                expected: "bool",
                actual: "int",
                ..
            })
        ));
    }

    #[test]
    fn test_missing_yes_branch() {
        // Start --> A{true} with only No edge
        let flowchart = Flowchart {
            direction: Direction::Td,
            nodes: vec![
                Node::Start { label: None },
                Node::Condition {
                    id: "A".to_string(),
                    condition: Expr::BoolLit { value: true },
                },
                Node::End { label: None },
            ],
            edges: vec![
                Edge {
                    from: "Start".to_string(),
                    to: "A".to_string(),
                    label: None,
                },
                Edge {
                    from: "A".to_string(),
                    to: "End".to_string(),
                    label: Some(EdgeLabel::No),
                },
            ],
        };
        let input = MockInputReader::new(vec![]);
        let output = MockOutputWriter::new();

        let mut interpreter = Interpreter::with_io(flowchart, input, output).unwrap();
        let result = interpreter.run();

        assert!(matches!(
            result,
            Err(RuntimeError::NoMatchingConditionEdge {
                node_id,
                condition_result: true,
            }) if node_id == "A"
        ));
    }

    #[test]
    fn test_missing_no_branch() {
        // Start --> A{false} with only Yes edge
        let flowchart = Flowchart {
            direction: Direction::Td,
            nodes: vec![
                Node::Start { label: None },
                Node::Condition {
                    id: "A".to_string(),
                    condition: Expr::BoolLit { value: false },
                },
                Node::End { label: None },
            ],
            edges: vec![
                Edge {
                    from: "Start".to_string(),
                    to: "A".to_string(),
                    label: None,
                },
                Edge {
                    from: "A".to_string(),
                    to: "End".to_string(),
                    label: Some(EdgeLabel::Yes),
                },
            ],
        };
        let input = MockInputReader::new(vec![]);
        let output = MockOutputWriter::new();

        let mut interpreter = Interpreter::with_io(flowchart, input, output).unwrap();
        let result = interpreter.run();

        assert!(matches!(
            result,
            Err(RuntimeError::NoMatchingConditionEdge {
                node_id,
                condition_result: false,
            }) if node_id == "A"
        ));
    }

    #[test]
    fn test_edge_to_nonexistent_node() {
        // Start --> NonExistent (node doesn't exist)
        let flowchart = Flowchart {
            direction: Direction::Td,
            nodes: vec![Node::Start { label: None }, Node::End { label: None }],
            edges: vec![Edge {
                from: "Start".to_string(),
                to: "NonExistent".to_string(),
                label: None,
            }],
        };
        let input = MockInputReader::new(vec![]);
        let output = MockOutputWriter::new();

        let mut interpreter = Interpreter::with_io(flowchart, input, output).unwrap();
        let result = interpreter.run();

        assert!(matches!(
            result,
            Err(RuntimeError::NodeNotFound { node_id }) if node_id == "NonExistent"
        ));
    }

    #[test]
    fn test_nested_conditions() {
        // Start --> A{x > 0} -->|Yes| B{x > 5} -->|Yes| C[print 'big']
        //                                       -->|No| D[print 'medium']
        //           A -->|No| E[print 'negative']
        // All paths --> End
        let flowchart = Flowchart {
            direction: Direction::Td,
            nodes: vec![
                Node::Start { label: None },
                Node::Process {
                    id: "Init".to_string(),
                    statements: vec![Statement::Assign {
                        variable: "x".to_string(),
                        value: Expr::IntLit { value: 3 },
                    }],
                },
                Node::Condition {
                    id: "A".to_string(),
                    condition: Expr::Binary {
                        op: crate::ast::BinaryOp::Gt,
                        left: Box::new(Expr::Variable {
                            name: "x".to_string(),
                        }),
                        right: Box::new(Expr::IntLit { value: 0 }),
                    },
                },
                Node::Condition {
                    id: "B".to_string(),
                    condition: Expr::Binary {
                        op: crate::ast::BinaryOp::Gt,
                        left: Box::new(Expr::Variable {
                            name: "x".to_string(),
                        }),
                        right: Box::new(Expr::IntLit { value: 5 }),
                    },
                },
                Node::Process {
                    id: "C".to_string(),
                    statements: vec![Statement::Println {
                        expr: Expr::StrLit {
                            value: "big".to_string(),
                        },
                    }],
                },
                Node::Process {
                    id: "D".to_string(),
                    statements: vec![Statement::Println {
                        expr: Expr::StrLit {
                            value: "medium".to_string(),
                        },
                    }],
                },
                Node::Process {
                    id: "E".to_string(),
                    statements: vec![Statement::Println {
                        expr: Expr::StrLit {
                            value: "negative".to_string(),
                        },
                    }],
                },
                Node::End { label: None },
            ],
            edges: vec![
                Edge {
                    from: "Start".to_string(),
                    to: "Init".to_string(),
                    label: None,
                },
                Edge {
                    from: "Init".to_string(),
                    to: "A".to_string(),
                    label: None,
                },
                Edge {
                    from: "A".to_string(),
                    to: "B".to_string(),
                    label: Some(EdgeLabel::Yes),
                },
                Edge {
                    from: "A".to_string(),
                    to: "E".to_string(),
                    label: Some(EdgeLabel::No),
                },
                Edge {
                    from: "B".to_string(),
                    to: "C".to_string(),
                    label: Some(EdgeLabel::Yes),
                },
                Edge {
                    from: "B".to_string(),
                    to: "D".to_string(),
                    label: Some(EdgeLabel::No),
                },
                Edge {
                    from: "C".to_string(),
                    to: "End".to_string(),
                    label: None,
                },
                Edge {
                    from: "D".to_string(),
                    to: "End".to_string(),
                    label: None,
                },
                Edge {
                    from: "E".to_string(),
                    to: "End".to_string(),
                    label: None,
                },
            ],
        };
        let input = MockInputReader::new(vec![]);
        let output = MockOutputWriter::new();

        let mut interpreter = Interpreter::with_io(flowchart, input, output).unwrap();
        interpreter.run().unwrap();

        // x = 3, so x > 0 (Yes) -> x > 5 (No) -> print "medium"
        assert_eq!(interpreter.output_writer.stdout, vec!["medium"]);
    }

    #[test]
    fn test_deep_nesting() {
        // Create a chain of 5 nested conditions
        // Start --> Init[x = 3] --> C1{x >= 1} -->|Yes| C2{x >= 2} -->|Yes| C3{x >= 3}
        //                                                                   -->|Yes| C4{x >= 4} -->|Yes| P1[print 'level 4']
        //                                                                                        -->|No| P2[print 'level 3']
        //                                                                   -->|No| P3[print 'level 2']
        //                                       -->|No| P4[print 'level 1']
        //           C1 -->|No| P5[print 'level 0']
        // All paths --> End
        let flowchart = Flowchart {
            direction: Direction::Td,
            nodes: vec![
                Node::Start { label: None },
                Node::Process {
                    id: "Init".to_string(),
                    statements: vec![Statement::Assign {
                        variable: "x".to_string(),
                        value: Expr::IntLit { value: 3 },
                    }],
                },
                Node::Condition {
                    id: "C1".to_string(),
                    condition: Expr::Binary {
                        op: crate::ast::BinaryOp::Ge,
                        left: Box::new(Expr::Variable {
                            name: "x".to_string(),
                        }),
                        right: Box::new(Expr::IntLit { value: 1 }),
                    },
                },
                Node::Condition {
                    id: "C2".to_string(),
                    condition: Expr::Binary {
                        op: crate::ast::BinaryOp::Ge,
                        left: Box::new(Expr::Variable {
                            name: "x".to_string(),
                        }),
                        right: Box::new(Expr::IntLit { value: 2 }),
                    },
                },
                Node::Condition {
                    id: "C3".to_string(),
                    condition: Expr::Binary {
                        op: crate::ast::BinaryOp::Ge,
                        left: Box::new(Expr::Variable {
                            name: "x".to_string(),
                        }),
                        right: Box::new(Expr::IntLit { value: 3 }),
                    },
                },
                Node::Condition {
                    id: "C4".to_string(),
                    condition: Expr::Binary {
                        op: crate::ast::BinaryOp::Ge,
                        left: Box::new(Expr::Variable {
                            name: "x".to_string(),
                        }),
                        right: Box::new(Expr::IntLit { value: 4 }),
                    },
                },
                Node::Process {
                    id: "P1".to_string(),
                    statements: vec![Statement::Println {
                        expr: Expr::StrLit {
                            value: "level 4".to_string(),
                        },
                    }],
                },
                Node::Process {
                    id: "P2".to_string(),
                    statements: vec![Statement::Println {
                        expr: Expr::StrLit {
                            value: "level 3".to_string(),
                        },
                    }],
                },
                Node::Process {
                    id: "P3".to_string(),
                    statements: vec![Statement::Println {
                        expr: Expr::StrLit {
                            value: "level 2".to_string(),
                        },
                    }],
                },
                Node::Process {
                    id: "P4".to_string(),
                    statements: vec![Statement::Println {
                        expr: Expr::StrLit {
                            value: "level 1".to_string(),
                        },
                    }],
                },
                Node::Process {
                    id: "P5".to_string(),
                    statements: vec![Statement::Println {
                        expr: Expr::StrLit {
                            value: "level 0".to_string(),
                        },
                    }],
                },
                Node::End { label: None },
            ],
            edges: vec![
                Edge {
                    from: "Start".to_string(),
                    to: "Init".to_string(),
                    label: None,
                },
                Edge {
                    from: "Init".to_string(),
                    to: "C1".to_string(),
                    label: None,
                },
                Edge {
                    from: "C1".to_string(),
                    to: "C2".to_string(),
                    label: Some(EdgeLabel::Yes),
                },
                Edge {
                    from: "C1".to_string(),
                    to: "P5".to_string(),
                    label: Some(EdgeLabel::No),
                },
                Edge {
                    from: "C2".to_string(),
                    to: "C3".to_string(),
                    label: Some(EdgeLabel::Yes),
                },
                Edge {
                    from: "C2".to_string(),
                    to: "P4".to_string(),
                    label: Some(EdgeLabel::No),
                },
                Edge {
                    from: "C3".to_string(),
                    to: "C4".to_string(),
                    label: Some(EdgeLabel::Yes),
                },
                Edge {
                    from: "C3".to_string(),
                    to: "P3".to_string(),
                    label: Some(EdgeLabel::No),
                },
                Edge {
                    from: "C4".to_string(),
                    to: "P1".to_string(),
                    label: Some(EdgeLabel::Yes),
                },
                Edge {
                    from: "C4".to_string(),
                    to: "P2".to_string(),
                    label: Some(EdgeLabel::No),
                },
                Edge {
                    from: "P1".to_string(),
                    to: "End".to_string(),
                    label: None,
                },
                Edge {
                    from: "P2".to_string(),
                    to: "End".to_string(),
                    label: None,
                },
                Edge {
                    from: "P3".to_string(),
                    to: "End".to_string(),
                    label: None,
                },
                Edge {
                    from: "P4".to_string(),
                    to: "End".to_string(),
                    label: None,
                },
                Edge {
                    from: "P5".to_string(),
                    to: "End".to_string(),
                    label: None,
                },
            ],
        };
        let input = MockInputReader::new(vec![]);
        let output = MockOutputWriter::new();

        let mut interpreter = Interpreter::with_io(flowchart, input, output).unwrap();
        interpreter.run().unwrap();

        // x = 3: C1(>=1 Yes) -> C2(>=2 Yes) -> C3(>=3 Yes) -> C4(>=4 No) -> P2 "level 3"
        assert_eq!(interpreter.output_writer.stdout, vec!["level 3"]);
    }
}
