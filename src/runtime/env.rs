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
//! The environment uses `FxHashMap` (a non-cryptographic hash map) for
//! O(1) average-case lookup and insertion. `FxHashMap` is faster than the
//! standard `HashMap` for short keys like variable names, at the cost of
//! no HashDoS resistance (acceptable since variable names come from trusted
//! `.mmd` source files).

use rustc_hash::FxHashMap;

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
    variables: FxHashMap<String, Value>,
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

    #[test]
    fn test_env_many_variables() {
        let mut env = Environment::new();

        // Set 1000 variables
        for i in 0..1000 {
            env.set(format!("var_{}", i), Value::Int(i));
        }

        // Verify all variables are correctly stored
        for i in 0..1000 {
            assert_eq!(env.get(&format!("var_{}", i)).unwrap(), &Value::Int(i));
        }

        // Verify different value types can coexist
        env.set("int_var".to_string(), Value::Int(42));
        env.set("str_var".to_string(), Value::Str("hello".to_string()));
        env.set("bool_var".to_string(), Value::Bool(true));

        assert_eq!(env.get("int_var").unwrap(), &Value::Int(42));
        assert_eq!(
            env.get("str_var").unwrap(),
            &Value::Str("hello".to_string())
        );
        assert_eq!(env.get("bool_var").unwrap(), &Value::Bool(true));
    }

    #[test]
    fn test_env_multiple_overwrite() {
        let mut env = Environment::new();

        // Overwrite variable many times
        for i in 0..100 {
            env.set("x".to_string(), Value::Int(i));
            assert_eq!(env.get("x").unwrap(), &Value::Int(i));
        }

        // Final value should be the last one set
        assert_eq!(env.get("x").unwrap(), &Value::Int(99));

        // Overwrite with different types
        env.set("y".to_string(), Value::Int(1));
        assert_eq!(env.get("y").unwrap(), &Value::Int(1));

        env.set("y".to_string(), Value::Str("changed".to_string()));
        assert_eq!(env.get("y").unwrap(), &Value::Str("changed".to_string()));

        env.set("y".to_string(), Value::Bool(false));
        assert_eq!(env.get("y").unwrap(), &Value::Bool(false));
    }

    #[test]
    fn test_env_special_char_names() {
        let mut env = Environment::new();

        // Variable names with underscores
        env.set("_leading_underscore".to_string(), Value::Int(1));
        env.set("trailing_underscore_".to_string(), Value::Int(2));
        env.set("__double__underscore__".to_string(), Value::Int(3));

        assert_eq!(env.get("_leading_underscore").unwrap(), &Value::Int(1));
        assert_eq!(env.get("trailing_underscore_").unwrap(), &Value::Int(2));
        assert_eq!(env.get("__double__underscore__").unwrap(), &Value::Int(3));

        // Variable names with numbers
        env.set("var123".to_string(), Value::Int(4));
        env.set("v1a2r3".to_string(), Value::Int(5));

        assert_eq!(env.get("var123").unwrap(), &Value::Int(4));
        assert_eq!(env.get("v1a2r3").unwrap(), &Value::Int(5));

        // Long variable names
        let long_name = "a".repeat(1000);
        env.set(long_name.clone(), Value::Int(6));
        assert_eq!(env.get(&long_name).unwrap(), &Value::Int(6));

        // Single character names
        env.set("a".to_string(), Value::Int(7));
        env.set("_".to_string(), Value::Int(8));

        assert_eq!(env.get("a").unwrap(), &Value::Int(7));
        assert_eq!(env.get("_").unwrap(), &Value::Int(8));
    }

    #[test]
    fn test_env_clone_independence() {
        let mut env1 = Environment::new();
        env1.set("x".to_string(), Value::Int(10));
        env1.set("y".to_string(), Value::Str("original".to_string()));

        // Clone the environment
        let mut env2 = env1.clone();

        // Verify clone has the same values
        assert_eq!(env2.get("x").unwrap(), &Value::Int(10));
        assert_eq!(env2.get("y").unwrap(), &Value::Str("original".to_string()));

        // Modify the clone
        env2.set("x".to_string(), Value::Int(20));
        env2.set("z".to_string(), Value::Bool(true));

        // Original should be unchanged
        assert_eq!(env1.get("x").unwrap(), &Value::Int(10));
        assert!(env1.get("z").is_err());

        // Clone should have new values
        assert_eq!(env2.get("x").unwrap(), &Value::Int(20));
        assert_eq!(env2.get("z").unwrap(), &Value::Bool(true));

        // Modify original
        env1.set("y".to_string(), Value::Str("modified".to_string()));

        // Original changed, clone unchanged
        assert_eq!(env1.get("y").unwrap(), &Value::Str("modified".to_string()));
        assert_eq!(env2.get("y").unwrap(), &Value::Str("original".to_string()));
    }
}
