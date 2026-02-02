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

    #[test]
    fn test_value_empty_string() {
        let empty = Value::Str(String::new());
        assert_eq!(empty.type_name(), "str");
        assert_eq!(empty.as_str(), Some(""));
        assert_eq!(empty.to_string(), "");

        let empty2 = Value::Str("".to_string());
        assert_eq!(empty, empty2);
    }

    #[test]
    fn test_value_unicode_string() {
        let japanese = Value::Str("こんにちは".to_string());
        assert_eq!(japanese.type_name(), "str");
        assert_eq!(japanese.as_str(), Some("こんにちは"));
        assert_eq!(japanese.to_string(), "こんにちは");

        let emoji = Value::Str("Hello 🌍🚀".to_string());
        assert_eq!(emoji.as_str(), Some("Hello 🌍🚀"));
        assert_eq!(emoji.to_string(), "Hello 🌍🚀");

        let mixed = Value::Str("日本語とEnglish混在".to_string());
        assert_eq!(mixed.as_str(), Some("日本語とEnglish混在"));
    }

    #[test]
    fn test_value_long_string() {
        let long_str = "a".repeat(10_000);
        let value = Value::Str(long_str.clone());
        assert_eq!(value.type_name(), "str");
        assert_eq!(value.as_str(), Some(long_str.as_str()));
        assert_eq!(value.to_string().len(), 10_000);

        let very_long = "x".repeat(1_000_000);
        let value2 = Value::Str(very_long.clone());
        assert_eq!(value2.as_str().map(|s| s.len()), Some(1_000_000));
    }

    #[test]
    fn test_value_special_chars() {
        let newline = Value::Str("line1\nline2".to_string());
        assert_eq!(newline.as_str(), Some("line1\nline2"));
        assert_eq!(newline.to_string(), "line1\nline2");

        let tab = Value::Str("col1\tcol2".to_string());
        assert_eq!(tab.as_str(), Some("col1\tcol2"));

        let backslash = Value::Str("path\\to\\file".to_string());
        assert_eq!(backslash.as_str(), Some("path\\to\\file"));

        let mixed_special = Value::Str("a\nb\tc\\d".to_string());
        assert_eq!(mixed_special.as_str(), Some("a\nb\tc\\d"));

        let carriage_return = Value::Str("line1\r\nline2".to_string());
        assert_eq!(carriage_return.as_str(), Some("line1\r\nline2"));

        let null_char = Value::Str("before\0after".to_string());
        assert_eq!(null_char.as_str(), Some("before\0after"));
    }

    #[test]
    fn test_value_partial_eq() {
        // Int == Int
        assert_eq!(Value::Int(42), Value::Int(42));
        assert_ne!(Value::Int(42), Value::Int(43));
        assert_eq!(Value::Int(0), Value::Int(0));
        assert_eq!(Value::Int(-1), Value::Int(-1));
        assert_eq!(Value::Int(i64::MAX), Value::Int(i64::MAX));
        assert_eq!(Value::Int(i64::MIN), Value::Int(i64::MIN));

        // Str == Str
        assert_eq!(
            Value::Str("hello".to_string()),
            Value::Str("hello".to_string())
        );
        assert_ne!(
            Value::Str("hello".to_string()),
            Value::Str("world".to_string())
        );
        assert_eq!(Value::Str(String::new()), Value::Str(String::new()));

        // Bool == Bool
        assert_eq!(Value::Bool(true), Value::Bool(true));
        assert_eq!(Value::Bool(false), Value::Bool(false));
        assert_ne!(Value::Bool(true), Value::Bool(false));

        // Int != Str
        assert_ne!(Value::Int(42), Value::Str("42".to_string()));
        assert_ne!(Value::Int(0), Value::Str("0".to_string()));

        // Int != Bool
        assert_ne!(Value::Int(1), Value::Bool(true));
        assert_ne!(Value::Int(0), Value::Bool(false));

        // Str != Bool
        assert_ne!(Value::Str("true".to_string()), Value::Bool(true));
        assert_ne!(Value::Str("false".to_string()), Value::Bool(false));
    }
}
