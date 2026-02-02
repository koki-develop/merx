use std::fmt;

/// Runtime value.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Str(String),
    Bool(bool),
}

impl Value {
    /// Returns the type name as a string.
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Int(_) => "int",
            Value::Str(_) => "str",
            Value::Bool(_) => "bool",
        }
    }

    /// Extracts the integer value. Returns `None` if not an integer.
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(n) => Some(*n),
            _ => None,
        }
    }

    /// Extracts the boolean value. Returns `None` if not a boolean.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Extracts the string value. Returns `None` if not a string.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }
}

impl fmt::Display for Value {
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
