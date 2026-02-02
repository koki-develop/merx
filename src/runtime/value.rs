//! Runtime value representation.
//!
//! This module defines the [`Value`] enum, which represents the three
//! fundamental data types in the merx language: integers, strings, and booleans.
//!
//! # Type System
//!
//! The merx language has a simple, dynamically-typed value system:
//!
//! | Type | Rust Representation | Example |
//! |------|---------------------|---------|
//! | `int` | `i64` | `42`, `-17` |
//! | `str` | `String` | `"hello"` |
//! | `bool` | `bool` | `true`, `false` |
//!
//! # Type Coercion
//!
//! Values do not implicitly coerce between types. Explicit casting is required
//! using the `as` syntax in the source language (e.g., `x as int`). Type casting
//! is handled internally during expression evaluation.
//!
//! # Display
//!
//! All value types implement [`Display`](std::fmt::Display) for output operations.
//! The display format matches the source language representation:
//!
//! - Integers: decimal notation (e.g., `42`)
//! - Strings: raw content without quotes (e.g., `hello`)
//! - Booleans: lowercase `true` or `false`

use std::fmt;

/// A runtime value in the merx language.
///
/// This enum represents the three fundamental types supported by the interpreter.
/// Values are immutable and can be cloned efficiently.
///
/// # Variants
///
/// - `Int` - A 64-bit signed integer
/// - `Str` - A UTF-8 string
/// - `Bool` - A boolean value
///
/// # Examples
///
/// ```
/// use merx::runtime::Value;
///
/// let int_val = Value::Int(42);
/// let str_val = Value::Str("hello".to_string());
/// let bool_val = Value::Bool(true);
///
/// assert_eq!(int_val.type_name(), "int");
/// assert_eq!(str_val.type_name(), "str");
/// assert_eq!(bool_val.type_name(), "bool");
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// A 64-bit signed integer value.
    ///
    /// Supports standard arithmetic operations: addition, subtraction,
    /// multiplication, division, and modulo.
    Int(i64),

    /// A UTF-8 string value.
    ///
    /// Strings are the result of string literals, user input, or type casts.
    Str(String),

    /// A boolean value.
    ///
    /// Used in condition nodes and logical operations.
    Bool(bool),
}

impl Value {
    /// Returns the type name of this value as a static string.
    ///
    /// This is used primarily for error messages when type mismatches occur.
    ///
    /// # Returns
    ///
    /// - `"int"` for [`Value::Int`]
    /// - `"str"` for [`Value::Str`]
    /// - `"bool"` for [`Value::Bool`]
    ///
    /// # Examples
    ///
    /// ```
    /// use merx::runtime::Value;
    ///
    /// assert_eq!(Value::Int(0).type_name(), "int");
    /// assert_eq!(Value::Str(String::new()).type_name(), "str");
    /// assert_eq!(Value::Bool(false).type_name(), "bool");
    /// ```
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Int(_) => "int",
            Value::Str(_) => "str",
            Value::Bool(_) => "bool",
        }
    }

    /// Attempts to extract an integer value.
    ///
    /// # Returns
    ///
    /// - `Some(n)` if this is a [`Value::Int`] containing `n`
    /// - `None` for all other variants
    ///
    /// # Examples
    ///
    /// ```
    /// use merx::runtime::Value;
    ///
    /// assert_eq!(Value::Int(42).as_int(), Some(42));
    /// assert_eq!(Value::Str("42".to_string()).as_int(), None);
    /// ```
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(n) => Some(*n),
            _ => None,
        }
    }

    /// Attempts to extract a boolean value.
    ///
    /// # Returns
    ///
    /// - `Some(b)` if this is a [`Value::Bool`] containing `b`
    /// - `None` for all other variants
    ///
    /// # Examples
    ///
    /// ```
    /// use merx::runtime::Value;
    ///
    /// assert_eq!(Value::Bool(true).as_bool(), Some(true));
    /// assert_eq!(Value::Int(1).as_bool(), None);
    /// ```
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Attempts to extract a string slice.
    ///
    /// # Returns
    ///
    /// - `Some(&str)` if this is a [`Value::Str`]
    /// - `None` for all other variants
    ///
    /// # Examples
    ///
    /// ```
    /// use merx::runtime::Value;
    ///
    /// assert_eq!(Value::Str("hello".to_string()).as_str(), Some("hello"));
    /// assert_eq!(Value::Int(42).as_str(), None);
    /// ```
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }
}

impl fmt::Display for Value {
    /// Formats the value for display output.
    ///
    /// The output format is designed for user-facing output (e.g., `print` statements):
    ///
    /// - Integers: decimal representation without formatting
    /// - Strings: raw content without surrounding quotes
    /// - Booleans: lowercase `true` or `false`
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Str(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_display() {
        assert_eq!(Value::Int(42).to_string(), "42");
        assert_eq!(Value::Str("hello".to_string()).to_string(), "hello");
        assert_eq!(Value::Bool(true).to_string(), "true");
        assert_eq!(Value::Bool(false).to_string(), "false");
    }

    #[test]
    fn test_value_type_name() {
        assert_eq!(Value::Int(0).type_name(), "int");
        assert_eq!(Value::Str(String::new()).type_name(), "str");
        assert_eq!(Value::Bool(false).type_name(), "bool");
    }

    #[test]
    fn test_value_as_int() {
        assert_eq!(Value::Int(42).as_int(), Some(42));
        assert_eq!(Value::Str("42".to_string()).as_int(), None);
        assert_eq!(Value::Bool(true).as_int(), None);
    }

    #[test]
    fn test_value_as_bool() {
        assert_eq!(Value::Bool(true).as_bool(), Some(true));
        assert_eq!(Value::Int(1).as_bool(), None);
        assert_eq!(Value::Str("true".to_string()).as_bool(), None);
    }

    #[test]
    fn test_value_as_str() {
        assert_eq!(Value::Str("hello".to_string()).as_str(), Some("hello"));
        assert_eq!(Value::Int(42).as_str(), None);
        assert_eq!(Value::Bool(true).as_str(), None);
    }
}
