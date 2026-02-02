use std::collections::HashMap;
use std::io;

use crate::ast::{Edge, EdgeLabel, Flowchart, Node};

use super::env::Environment;
use super::error::RuntimeError;
use super::eval::{eval_expr, InputReader, StdinReader};
use super::exec::{exec_statement, OutputWriter, StdioWriter};

/// The interpreter.
pub struct Interpreter<R: InputReader, W: OutputWriter> {
    /// Map from node ID to node.
    nodes: HashMap<String, Node>,
    /// Map from node ID to outgoing edges.
    outgoing_edges: HashMap<String, Vec<Edge>>,
    /// Current node ID.
    current_node_id: String,
    /// Variable environment.
    env: Environment,
    /// Input reader.
    input_reader: R,
    /// Output writer.
    output_writer: W,
}

impl Interpreter<StdinReader<io::BufReader<io::Stdin>>, StdioWriter> {
    /// Creates an interpreter with default I/O (stdin/stdout).
    pub fn new(flowchart: Flowchart) -> Result<Self, RuntimeError> {
        Self::with_io(flowchart, StdinReader::new(), StdioWriter::new())
    }
}

impl<R: InputReader, W: OutputWriter> Interpreter<R, W> {
    /// Creates an interpreter with custom I/O.
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
                Node::Start => start_count += 1,
                Node::End => end_count += 1,
                _ => {}
            }
            nodes.insert(id, node);
        }

        // Validate Start/End nodes
        if start_count == 0 {
            return Err(RuntimeError::MissingStartNode);
        }
        if start_count > 1 {
            return Err(RuntimeError::MultipleStartNodes);
        }
        if end_count == 0 {
            return Err(RuntimeError::MissingEndNode);
        }
        if end_count > 1 {
            return Err(RuntimeError::MultipleEndNodes);
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

    /// Runs the program.
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
                Node::Start => {
                    // Move to the next node from Start
                    self.move_to_next()?;
                }
                Node::End => {
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

    /// Moves to the next node (normal edge).
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

    /// Moves to the next node based on the condition result.
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

        fn write_stderr(&mut self, s: &str) {
            self.stderr.push(s.to_string());
        }
    }

    fn create_simple_flowchart() -> Flowchart {
        // Start --> A[print 'hello'] --> End
        Flowchart {
            direction: Direction::Td,
            nodes: vec![
                Node::Start,
                Node::Process {
                    id: "A".to_string(),
                    statements: vec![Statement::Print {
                        expr: Expr::StrLit {
                            value: "hello".to_string(),
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
                        op: crate::ast::BinaryOp::Gt,
                        left: Box::new(Expr::Variable {
                            name: "x".to_string(),
                        }),
                        right: Box::new(Expr::IntLit { value: 3 }),
                    },
                },
                Node::Process {
                    id: "C".to_string(),
                    statements: vec![Statement::Print {
                        expr: Expr::StrLit {
                            value: "big".to_string(),
                        },
                    }],
                },
                Node::Process {
                    id: "D".to_string(),
                    statements: vec![Statement::Print {
                        expr: Expr::StrLit {
                            value: "small".to_string(),
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
            nodes: vec![Node::End],
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
            nodes: vec![Node::Start],
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
            nodes: vec![Node::Start, Node::End],
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
                Node::Start,
                Node::Process {
                    id: "A".to_string(),
                    statements: vec![Statement::Error {
                        message: Expr::StrLit {
                            value: "test error".to_string(),
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
                Node::Start,
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
                        Statement::Print {
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
}
