use std::collections::HashMap;

use super::error::RuntimeError;
use super::value::Value;

/// Variable environment.
#[derive(Debug, Clone, Default)]
pub struct Environment {
    variables: HashMap<String, Value>,
}

impl Environment {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets or updates a variable.
    pub fn set(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    /// Gets a variable. Returns an error if undefined.
    pub fn get(&self, name: &str) -> Result<&Value, RuntimeError> {
        self.variables
            .get(name)
            .ok_or_else(|| RuntimeError::UndefinedVariable {
                name: name.to_string(),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_and_get() {
        let mut env = Environment::new();
        env.set("x".to_string(), Value::Int(42));
        assert_eq!(env.get("x").unwrap(), &Value::Int(42));
    }

    #[test]
    fn test_get_undefined() {
        let env = Environment::new();
        let result = env.get("x");
        assert!(matches!(
            result,
            Err(RuntimeError::UndefinedVariable { name }) if name == "x"
        ));
    }

    #[test]
    fn test_overwrite() {
        let mut env = Environment::new();
        env.set("x".to_string(), Value::Int(1));
        env.set("x".to_string(), Value::Int(2));
        assert_eq!(env.get("x").unwrap(), &Value::Int(2));
    }
}
