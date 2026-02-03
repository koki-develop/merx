use super::error::RuntimeError;
use super::eval::InputReader;
use super::exec::OutputWriter;

/// Mock input reader for testing.
///
/// Returns pre-configured lines in order, then returns an error when exhausted.
pub(crate) struct MockInputReader {
    lines: Vec<String>,
    index: usize,
}

impl MockInputReader {
    pub(crate) fn new(lines: Vec<&str>) -> Self {
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
///
/// Captures stdout and stderr output into separate vectors.
pub(crate) struct MockOutputWriter {
    pub stdout: Vec<String>,
    pub stderr: Vec<String>,
}

impl MockOutputWriter {
    pub(crate) fn new() -> Self {
        Self {
            stdout: Vec::new(),
            stderr: Vec::new(),
        }
    }
}

impl OutputWriter for MockOutputWriter {
    fn write_stdout(&mut self, s: &str) -> Result<(), RuntimeError> {
        self.stdout.push(s.to_string());
        Ok(())
    }

    fn write_stdout_no_newline(&mut self, s: &str) -> Result<(), RuntimeError> {
        self.stdout.push(s.to_string());
        Ok(())
    }

    fn write_stderr(&mut self, s: &str) -> Result<(), RuntimeError> {
        self.stderr.push(s.to_string());
        Ok(())
    }
}

/// Output writer that always fails, for testing error propagation.
pub(crate) struct FailingOutputWriter;

impl OutputWriter for FailingOutputWriter {
    fn write_stdout(&mut self, _s: &str) -> Result<(), RuntimeError> {
        Err(RuntimeError::IoError {
            message: "stdout write failed".to_string(),
        })
    }

    fn write_stdout_no_newline(&mut self, _s: &str) -> Result<(), RuntimeError> {
        Err(RuntimeError::IoError {
            message: "stdout write failed".to_string(),
        })
    }

    fn write_stderr(&mut self, _s: &str) -> Result<(), RuntimeError> {
        Err(RuntimeError::IoError {
            message: "stderr write failed".to_string(),
        })
    }
}
