//! Integration tests for the merx Mermaid flowchart interpreter.
//!
//! These tests verify end-to-end functionality using actual `.mmd` files,
//! testing the complete pipeline from parsing to execution.

use merx::parser;
use merx::runtime::{InputReader, Interpreter, OutputWriter, RuntimeError};

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

    fn empty() -> Self {
        Self::new(vec![])
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

/// Helper function to run a flowchart from source code.
fn run_flowchart(source: &str) -> Result<(Vec<String>, Vec<String>), String> {
    run_flowchart_with_input(source, vec![])
}

/// Helper function to run a flowchart with mock input.
fn run_flowchart_with_input(
    source: &str,
    input_lines: Vec<&str>,
) -> Result<(Vec<String>, Vec<String>), String> {
    let flowchart = parser::parse(source).map_err(|e| e.to_string())?;

    let input = MockInputReader::new(input_lines);
    let output = MockOutputWriter::new();

    let mut interpreter =
        Interpreter::with_io(flowchart, input, output).map_err(|e| e.to_string())?;

    interpreter.run().map_err(|e| e.to_string())?;

    let output = interpreter.into_output_writer();
    Ok((output.stdout, output.stderr))
}

/// Helper to run a flowchart expecting it to succeed with captured output.
#[allow(dead_code)]
fn run_flowchart_expect_output(source: &str) -> MockOutputWriter {
    let flowchart = parser::parse(source).expect("Failed to parse");
    let input = MockInputReader::empty();
    let output = MockOutputWriter::new();
    let mut interpreter =
        Interpreter::with_io(flowchart, input, output).expect("Failed to create interpreter");
    interpreter.run().expect("Failed to run");
    interpreter.into_output_writer()
}

// =============================================================================
// Valid flowchart tests
// =============================================================================

mod valid_flowcharts {
    use super::*;

    #[test]
    fn test_hello_world() {
        let source = include_str!("fixtures/valid/hello_world.mmd");
        let (stdout, stderr) = run_flowchart(source).expect("Should execute successfully");

        assert_eq!(stdout, vec!["Hello, World!"]);
        assert!(stderr.is_empty());
    }

    #[test]
    fn test_calculator() {
        let source = include_str!("fixtures/valid/calculator.mmd");
        let (stdout, stderr) = run_flowchart(source).expect("Should execute successfully");

        // x = 10, y = 3
        // sum = 13, diff = 7, prod = 30, quot = 3, rem = 1
        assert_eq!(stdout, vec!["13", "7", "30", "3", "1"]);
        assert!(stderr.is_empty());
    }

    #[test]
    fn test_fizzbuzz() {
        let source = include_str!("fixtures/valid/fizzbuzz.mmd");
        let (stdout, stderr) = run_flowchart(source).expect("Should execute successfully");

        let expected = vec![
            "1", "2", "Fizz", "4", "Buzz", "Fizz", "7", "8", "Fizz", "Buzz", "11", "Fizz", "13",
            "14", "FizzBuzz",
        ];
        assert_eq!(stdout, expected);
        assert!(stderr.is_empty());
    }

    #[test]
    fn test_conditional() {
        let source = include_str!("fixtures/valid/conditional.mmd");
        let (stdout, stderr) = run_flowchart(source).expect("Should execute successfully");

        // x = 5, so x > 10 is false, x > 0 is true
        assert_eq!(stdout, vec!["small but positive"]);
        assert!(stderr.is_empty());
    }

    #[test]
    fn test_loop_counter() {
        let source = include_str!("fixtures/valid/loop_counter.mmd");
        let (stdout, stderr) = run_flowchart(source).expect("Should execute successfully");

        // Sum of 1 + 2 + 3 + 4 + 5 = 15
        assert_eq!(stdout, vec!["15"]);
        assert!(stderr.is_empty());
    }

    #[test]
    fn test_string_operations() {
        let source = include_str!("fixtures/valid/string_operations.mmd");
        let (stdout, stderr) = run_flowchart(source).expect("Should execute successfully");

        assert_eq!(stdout, vec!["hello", "42"]);
        assert!(stderr.is_empty());
    }

    #[test]
    fn test_boolean_logic() {
        let source = include_str!("fixtures/valid/boolean_logic.mmd");
        let (stdout, stderr) = run_flowchart(source).expect("Should execute successfully");

        // a = true, b = false
        // a && b = false, a || b = true
        assert_eq!(stdout, vec!["at least one true"]);
        assert!(stderr.is_empty());
    }

    #[test]
    fn test_error_output() {
        let source = include_str!("fixtures/valid/error_output.mmd");
        let (stdout, stderr) = run_flowchart(source).expect("Should execute successfully");

        assert!(stdout.is_empty());
        assert_eq!(stderr, vec!["This is an error message"]);
    }
}

// =============================================================================
// Invalid flowchart tests - Parse errors
// =============================================================================

mod invalid_flowcharts {
    use super::*;

    #[test]
    fn test_invalid_syntax() {
        let source = include_str!("fixtures/invalid/invalid_syntax.mmd");
        let result = parser::parse(source);

        assert!(result.is_err(), "Should fail to parse invalid syntax");
    }

    #[test]
    fn test_missing_yes_edge() {
        let source = include_str!("fixtures/invalid/missing_yes_edge.mmd");
        let result = parser::parse(source);

        assert!(result.is_err(), "Should fail with missing Yes edge");
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("missing 'Yes' edge"),
            "Error should mention missing Yes edge: {}",
            err
        );
    }

    #[test]
    fn test_missing_no_edge() {
        let source = include_str!("fixtures/invalid/missing_no_edge.mmd");
        let result = parser::parse(source);

        assert!(result.is_err(), "Should fail with missing No edge");
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("missing 'No' edge"),
            "Error should mention missing No edge: {}",
            err
        );
    }

    #[test]
    fn test_multiple_yes_edges() {
        let source = include_str!("fixtures/invalid/multiple_yes_edges.mmd");
        let result = parser::parse(source);

        assert!(result.is_err(), "Should fail with multiple Yes edges");
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("multiple 'Yes' edges"),
            "Error should mention multiple Yes edges: {}",
            err
        );
    }

    #[test]
    fn test_end_with_outgoing_edge() {
        let source = include_str!("fixtures/invalid/end_with_outgoing_edge.mmd");
        let result = parser::parse(source);

        assert!(result.is_err(), "Should fail when End has outgoing edge");
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("End node cannot have outgoing"),
            "Error should mention End node outgoing edge: {}",
            err
        );
    }
}

// =============================================================================
// Runtime error tests
// =============================================================================

mod runtime_errors {
    use super::*;

    #[test]
    fn test_missing_start_node_runtime() {
        // Parser allows missing Start, but runtime should fail
        let source = include_str!("fixtures/invalid/missing_start.mmd");

        // First verify it parses
        let flowchart = parser::parse(source).expect("Should parse successfully");

        // Then verify runtime fails
        let input = MockInputReader::empty();
        let output = MockOutputWriter::new();
        let result = Interpreter::with_io(flowchart, input, output);

        assert!(
            result.is_err(),
            "Should fail at runtime with missing Start node"
        );
        match result {
            Err(RuntimeError::MissingStartNode) => {}
            Err(e) => panic!("Should be MissingStartNode error, got: {:?}", e),
            Ok(_) => panic!("Should have failed"),
        }
    }

    #[test]
    fn test_missing_end_node_runtime() {
        // Parser allows missing End, but runtime should fail
        let source = include_str!("fixtures/invalid/missing_end.mmd");

        // First verify it parses
        let flowchart = parser::parse(source).expect("Should parse successfully");

        // Then verify runtime fails
        let input = MockInputReader::empty();
        let output = MockOutputWriter::new();
        let result = Interpreter::with_io(flowchart, input, output);

        assert!(
            result.is_err(),
            "Should fail at runtime with missing End node"
        );
        match result {
            Err(RuntimeError::MissingEndNode) => {}
            Err(e) => panic!("Should be MissingEndNode error, got: {:?}", e),
            Ok(_) => panic!("Should have failed"),
        }
    }

    #[test]
    fn test_undefined_variable() {
        let source = r#"flowchart TD
    Start --> A[println x]
    A --> End
"#;
        let result = run_flowchart(source);
        assert!(result.is_err(), "Should fail with undefined variable");
        assert!(
            result.unwrap_err().contains("Undefined"),
            "Error should mention undefined variable"
        );
    }

    #[test]
    fn test_type_error_arithmetic() {
        let source = r#"flowchart TD
    Start --> A[x = 'hello' + 1]
    A --> End
"#;
        let result = run_flowchart(source);
        assert!(result.is_err(), "Should fail with type error");
    }

    #[test]
    fn test_division_by_zero() {
        let source = r#"flowchart TD
    Start --> A[x = 10 / 0]
    A --> End
"#;
        let result = run_flowchart(source);
        assert!(result.is_err(), "Should fail with division by zero");
        assert!(
            result.unwrap_err().contains("Division by zero"),
            "Error should mention division by zero"
        );
    }

    #[test]
    fn test_modulo_by_zero() {
        let source = r#"flowchart TD
    Start --> A[x = 10 % 0]
    A --> End
"#;
        let result = run_flowchart(source);
        assert!(result.is_err(), "Should fail with modulo by zero");
        assert!(
            result.unwrap_err().contains("Division by zero"),
            "Error should mention division by zero"
        );
    }

    #[test]
    fn test_condition_non_bool() {
        let source = r#"flowchart TD
    Start --> A{42?}
    A -->|Yes| End
    A -->|No| End
"#;
        let result = run_flowchart(source);
        assert!(result.is_err(), "Should fail when condition is not bool");
        let err = result.unwrap_err();
        assert!(
            err.contains("Type error"),
            "Error should be Type error, got: {}",
            err
        );
    }

    #[test]
    fn test_invalid_cast_string_to_int() {
        let source = r#"flowchart TD
    Start --> A[x = 'not a number' as int]
    A --> End
"#;
        let result = run_flowchart(source);
        assert!(result.is_err(), "Should fail with invalid cast");
    }
}

// =============================================================================
// Input handling tests
// =============================================================================

mod input_handling {
    use super::*;

    #[test]
    fn test_simple_input() {
        let source = r#"flowchart TD
    Start --> A[x = input]
    A --> B[println x]
    B --> End
"#;
        let (stdout, _) =
            run_flowchart_with_input(source, vec!["hello"]).expect("Should execute successfully");

        assert_eq!(stdout, vec!["hello"]);
    }

    #[test]
    fn test_input_as_int() {
        let source = r#"flowchart TD
    Start --> A[x = input as int]
    A --> B[y = x * 2]
    B --> C[println y]
    C --> End
"#;
        let (stdout, _) =
            run_flowchart_with_input(source, vec!["21"]).expect("Should execute successfully");

        assert_eq!(stdout, vec!["42"]);
    }

    #[test]
    fn test_multiple_inputs() {
        let source = r#"flowchart TD
    Start --> A[a = input as int; b = input as int]
    A --> B[sum = a + b]
    B --> C[println sum]
    C --> End
"#;
        let (stdout, _) = run_flowchart_with_input(source, vec!["10", "20"])
            .expect("Should execute successfully");

        assert_eq!(stdout, vec!["30"]);
    }
}

// =============================================================================
// Complex flowchart tests
// =============================================================================

mod complex_flowcharts {
    use super::*;

    #[test]
    fn test_nested_conditions() {
        let source = r#"flowchart TD
    Start --> Init[x = 7]
    Init --> A{x > 10?}
    A -->|Yes| Big[println 'big']
    A -->|No| B{x > 5?}
    B -->|Yes| Medium[println 'medium']
    B -->|No| C{x > 0?}
    C -->|Yes| Small[println 'small']
    C -->|No| Zero[println 'zero or negative']
    Big --> End
    Medium --> End
    Small --> End
    Zero --> End
"#;
        let (stdout, _) = run_flowchart(source).expect("Should execute successfully");
        assert_eq!(stdout, vec!["medium"]);
    }

    #[test]
    fn test_loop_with_early_exit() {
        let source = r#"flowchart TD
    Start --> Init[i = 1]
    Init --> Loop{i <= 10?}
    Loop -->|No| Done[println 'done']
    Done --> End
    Loop -->|Yes| Check{i == 5?}
    Check -->|Yes| Break[println 'found 5']
    Break --> End
    Check -->|No| Print[println i]
    Print --> Inc[i = i + 1]
    Inc --> Loop
"#;
        let (stdout, _) = run_flowchart(source).expect("Should execute successfully");
        assert_eq!(stdout, vec!["1", "2", "3", "4", "found 5"]);
    }

    #[test]
    fn test_factorial_like_computation() {
        // Compute 5! = 120
        let source = r#"flowchart TD
    Start --> Init[n = 5; result = 1]
    Init --> Loop{n > 0?}
    Loop -->|No| Print[println result]
    Print --> End
    Loop -->|Yes| Mul[result = result * n; n = n - 1]
    Mul --> Loop
"#;
        let (stdout, _) = run_flowchart(source).expect("Should execute successfully");
        assert_eq!(stdout, vec!["120"]);
    }

    #[test]
    fn test_comparison_operators() {
        let source = r#"flowchart TD
    Start --> Init[x = 5; y = 5]
    Init --> A{x == y?}
    A -->|Yes| EqTrue[println 'eq']
    A -->|No| EqFalse[println 'neq']
    EqTrue --> B{x != y?}
    EqFalse --> B
    B -->|Yes| NeTrue[println 'ne']
    B -->|No| NeFalse[println 'nne']
    NeTrue --> C{x < y?}
    NeFalse --> C
    C -->|Yes| LtTrue[println 'lt']
    C -->|No| LtFalse[println 'nlt']
    LtTrue --> D{x <= y?}
    LtFalse --> D
    D -->|Yes| LeTrue[println 'le']
    D -->|No| LeFalse[println 'nle']
    LeTrue --> End
    LeFalse --> End
"#;
        let (stdout, _) = run_flowchart(source).expect("Should execute successfully");
        // x == y: true -> 'eq'
        // x != y: false -> 'nne'
        // x < y: false -> 'nlt'
        // x <= y: true -> 'le'
        assert_eq!(stdout, vec!["eq", "nne", "nlt", "le"]);
    }

    #[test]
    fn test_unary_operators() {
        let source = r#"flowchart TD
    Start --> Init[x = 5; b = true]
    Init --> A[y = -x]
    A --> B[println y]
    B --> C{!b?}
    C -->|Yes| D[println 'not true']
    C -->|No| E[println 'still true']
    D --> End
    E --> End
"#;
        let (stdout, _) = run_flowchart(source).expect("Should execute successfully");
        assert_eq!(stdout, vec!["-5", "still true"]);
    }
}

// =============================================================================
// Direction tests (parsing only, direction doesn't affect execution)
// =============================================================================

mod direction_tests {
    use super::*;
    use merx::ast::Direction;

    #[test]
    fn test_all_directions_parse() {
        let directions = [
            ("TD", Direction::Td),
            ("TB", Direction::Tb),
            ("LR", Direction::Lr),
            ("RL", Direction::Rl),
            ("BT", Direction::Bt),
        ];

        for (dir_str, expected_dir) in directions {
            let source = format!(
                r#"flowchart {}
    Start --> End
"#,
                dir_str
            );
            let flowchart = parser::parse(&source).expect("Should parse");
            assert!(
                matches!(flowchart.direction, ref d if std::mem::discriminant(d) == std::mem::discriminant(&expected_dir)),
                "Direction should be {:?} for {}",
                expected_dir,
                dir_str
            );
        }
    }
}
