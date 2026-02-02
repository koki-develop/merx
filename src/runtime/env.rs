//! Variable environment for runtime execution.
//!
//! This module provides [`Environment`], a simple key-value store for
//! variable bindings during program execution.
//!
//! # Scoping
//!
//! The merx language uses a single global scope. All variables are visible
//! from the point of assignment until program termination. There is no
//! block scoping or variable shadowing.
//!
//! # Variable Lifecycle
//!
//! 1. Variables come into existence via assignment statements
//! 2. Variables can be reassigned at any time
//! 3. Variables persist until program ends
//! 4. Accessing an undefined variable is a runtime error
//!
//! # Implementation
//!
//! The environment uses a [`HashMap`] for O(1) average-case lookup
//! and insertion.

use std::collections::HashMap;

use super::error::RuntimeError;
use super::value::Value;

/// Storage for variable bindings during execution.
///
/// The environment maintains a mapping from variable names to their
/// current values. It provides operations to set (create or update)
/// and get variables.
///
/// # Examples
///
/// ```
/// use merx::runtime::{Environment, Value};
///
/// let mut env = Environment::new();
///
/// // Create a new variable
/// env.set("x".to_string(), Value::Int(42));
///
/// // Read its value
/// assert_eq!(env.get("x").unwrap(), &Value::Int(42));
///
/// // Update it
/// env.set("x".to_string(), Value::Int(100));
/// assert_eq!(env.get("x").unwrap(), &Value::Int(100));
/// ```
///
/// # Errors
///
/// Attempting to read an undefined variable returns
/// [`RuntimeError::UndefinedVariable`].
///
/// ```
/// use merx::runtime::{Environment, RuntimeError};
///
/// let env = Environment::new();
/// let result = env.get("undefined");
/// assert!(matches!(result, Err(RuntimeError::UndefinedVariable { .. })));
/// ```
#[derive(Debug, Clone, Default)]
pub struct Environment {
    /// Map from variable names to their current values.
    variables: HashMap<String, Value>,
}

impl Environment {
    /// Creates an empty environment with no variable bindings.
    ///
    /// # Examples
    ///
    /// ```
    /// use merx::runtime::Environment;
    ///
    /// let env = Environment::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets or updates a variable binding.
    ///
    /// If the variable already exists, its value is replaced.
    /// If it doesn't exist, a new binding is created.
    ///
    /// # Arguments
    ///
    /// * `name` - The variable identifier
    /// * `value` - The value to assign
    ///
    /// # Examples
    ///
    /// ```
    /// use merx::runtime::{Environment, Value};
    ///
    /// let mut env = Environment::new();
    /// env.set("counter".to_string(), Value::Int(0));
    /// env.set("counter".to_string(), Value::Int(1)); // Updates existing
    /// ```
    pub fn set(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    /// Retrieves a variable's value by name.
    ///
    /// # Arguments
    ///
    /// * `name` - The variable identifier to look up
    ///
    /// # Returns
    ///
    /// - `Ok(&Value)` if the variable exists
    /// - `Err(RuntimeError::UndefinedVariable)` if not found
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError::UndefinedVariable`] if no variable with
    /// the given name has been set.
    ///
    /// # Examples
    ///
    /// ```
    /// use merx::runtime::{Environment, Value};
    ///
    /// let mut env = Environment::new();
    /// env.set("x".to_string(), Value::Int(42));
    ///
    /// // Successful lookup
    /// let value = env.get("x").unwrap();
    /// assert_eq!(value, &Value::Int(42));
    ///
    /// // Undefined variable
    /// let result = env.get("undefined");
    /// assert!(result.is_err());
    /// ```
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
