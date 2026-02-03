//! Expression types for computations and values.
//!
//! Expressions form the computational core of the merx language. They can
//! represent literals, variables, operators, type casts, and user input.
//! Expressions are used in assignments, print statements, error statements,
//! and condition nodes.

use std::fmt;

/// An expression that can be evaluated to produce a value.
///
/// Expressions are the building blocks for computations in merx. They can
/// be nested arbitrarily to form complex calculations.
///
/// # Expression Types
///
/// | Variant | Mermaid Syntax | Description |
/// |---------|----------------|-------------|
/// | [`IntLit`](Expr::IntLit) | `42` | Integer literal |
/// | [`StrLit`](Expr::StrLit) | `'hello'` | String literal (single quotes) |
/// | [`BoolLit`](Expr::BoolLit) | `true`, `false` | Boolean literal |
/// | [`Variable`](Expr::Variable) | `x` | Variable reference |
/// | [`Input`](Expr::Input) | `input` | Read from stdin |
/// | [`Unary`](Expr::Unary) | `-x`, `!b` | Unary operation |
/// | [`Binary`](Expr::Binary) | `x + y` | Binary operation |
/// | [`Cast`](Expr::Cast) | `x as int` | Type conversion |
///
/// # Operator Precedence
///
/// From highest to lowest:
/// 1. Unary: `-`, `!`
/// 2. Multiplicative: `*`, `/`, `%`
/// 3. Additive: `+`, `-`
/// 4. Comparison: `<`, `<=`, `>`, `>=`
/// 5. Equality: `==`, `!=`
/// 6. Logical AND: `&&`
/// 7. Logical OR: `||`
///
/// # Examples
///
/// ```text
/// x = 5                    // IntLit assigned to variable
/// y = x + 3                // Binary operation
/// z = -y                   // Unary negation
/// s = input as int         // Input with type cast
/// b = x > 0 && y < 10      // Compound boolean expression
/// ```
///
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// An integer literal.
    ///
    /// Represents a 64-bit signed integer value.
    ///
    /// # Mermaid Syntax
    ///
    /// ```text
    /// 42
    /// -17
    /// 0
    /// ```
    IntLit {
        /// The integer value.
        value: i64,
    },

    /// A string literal.
    ///
    /// Strings are enclosed in single quotes in the source syntax.
    ///
    /// # Mermaid Syntax
    ///
    /// ```text
    /// 'hello'
    /// 'Hello, World!'
    /// ''
    /// ```
    StrLit {
        /// The string value (without enclosing quotes).
        value: String,
    },

    /// A boolean literal.
    ///
    /// # Mermaid Syntax
    ///
    /// ```text
    /// true
    /// false
    /// ```
    BoolLit {
        /// The boolean value.
        value: bool,
    },

    /// A variable reference.
    ///
    /// Variables must be assigned before use. Variable names follow identifier
    /// rules: start with a letter or underscore, followed by letters, digits,
    /// or underscores.
    ///
    /// # Mermaid Syntax
    ///
    /// ```text
    /// x
    /// myVariable
    /// _count
    /// ```
    Variable {
        /// The variable name.
        name: String,
    },

    /// Read a line from standard input.
    ///
    /// Returns the input as a string value. The trailing newline is stripped.
    /// Can be combined with type casting to read integers.
    ///
    /// # Mermaid Syntax
    ///
    /// ```text
    /// input           // Read as string
    /// input as int    // Read and convert to integer
    /// ```
    Input,

    /// A unary operation.
    ///
    /// Applies a unary operator to a single operand.
    ///
    /// # Mermaid Syntax
    ///
    /// ```text
    /// -x      // Negation
    /// !b      // Logical NOT
    /// ```
    Unary {
        /// The unary operator to apply.
        op: UnaryOp,
        /// The operand expression.
        operand: Box<Expr>,
    },

    /// A binary operation.
    ///
    /// Applies a binary operator to two operands.
    ///
    /// # Mermaid Syntax
    ///
    /// ```text
    /// x + y
    /// a * b
    /// x == y
    /// a && b
    /// ```
    Binary {
        /// The binary operator to apply.
        op: BinaryOp,
        /// The left operand expression.
        left: Box<Expr>,
        /// The right operand expression.
        right: Box<Expr>,
    },

    /// A type cast expression.
    ///
    /// Converts a value to the specified type. Supported conversions:
    /// - `int as str`: Integer to string (decimal representation)
    /// - `str as int`: String to integer (parse decimal, may fail at runtime)
    /// - `bool as str`: Boolean to string (`"true"` or `"false"`)
    ///
    /// Note: `bool as int` is NOT supported and will produce a runtime error.
    ///
    /// # Mermaid Syntax
    ///
    /// ```text
    /// x as int
    /// y as str
    /// ```
    ///
    /// # Errors
    ///
    /// - Casting a non-numeric string to `int` produces a runtime error
    /// - Casting `bool` to `int` produces a runtime error
    Cast {
        /// The expression to cast.
        expr: Box<Expr>,
        /// The target type for the cast.
        target_type: TypeName,
    },
}

/// A unary operator.
///
/// Unary operators take a single operand and produce a result.
///
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp {
    /// Logical NOT (`!`).
    ///
    /// Inverts a boolean value. Requires a boolean operand.
    ///
    /// # Truth Table
    ///
    /// | Input | Output |
    /// |-------|--------|
    /// | true  | false  |
    /// | false | true   |
    Not,

    /// Arithmetic negation (`-`).
    ///
    /// Negates an integer value. Requires an integer operand.
    Neg,
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOp::Not => write!(f, "logical NOT (!)"),
            UnaryOp::Neg => write!(f, "negation (-)"),
        }
    }
}

/// A binary operator.
///
/// Binary operators take two operands and produce a result. Operators are
/// listed roughly in order of precedence (highest first within groups).
///
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOp {
    /// Addition / concatenation (`+`).
    ///
    /// - Integer + Integer → Integer (wrapping on overflow)
    /// - String + String → String (concatenation)
    ///
    /// Mixed-type operands (e.g., `int + str`) produce a runtime type error.
    Add,

    /// Subtraction (`-`).
    ///
    /// Integer - Integer → Integer (wrapping on overflow)
    Sub,

    /// Multiplication (`*`).
    ///
    /// Integer * Integer → Integer (wrapping on overflow)
    Mul,

    /// Division (`/`).
    ///
    /// Integer / Integer → Integer (truncating division)
    ///
    /// # Errors
    ///
    /// Division by zero produces a runtime error.
    Div,

    /// Modulo (`%`).
    ///
    /// Integer % Integer → Integer (remainder)
    ///
    /// # Errors
    ///
    /// Modulo by zero produces a runtime error.
    Mod,

    /// Equality (`==`).
    ///
    /// Compares two values of the same type for equality.
    /// Returns a boolean.
    Eq,

    /// Inequality (`!=`).
    ///
    /// Compares two values of the same type for inequality.
    /// Returns a boolean.
    Ne,

    /// Less than (`<`).
    ///
    /// Compares two integers. Returns a boolean.
    Lt,

    /// Less than or equal (`<=`).
    ///
    /// Compares two integers. Returns a boolean.
    Le,

    /// Greater than (`>`).
    ///
    /// Compares two integers. Returns a boolean.
    Gt,

    /// Greater than or equal (`>=`).
    ///
    /// Compares two integers. Returns a boolean.
    Ge,

    /// Logical AND (`&&`).
    ///
    /// Both operands are always evaluated (no short-circuit evaluation).
    ///
    /// Boolean && Boolean → Boolean
    And,

    /// Logical OR (`||`).
    ///
    /// Both operands are always evaluated (no short-circuit evaluation).
    ///
    /// Boolean || Boolean → Boolean
    Or,
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOp::Add => write!(f, "addition (+)"),
            BinaryOp::Sub => write!(f, "subtraction (-)"),
            BinaryOp::Mul => write!(f, "multiplication (*)"),
            BinaryOp::Div => write!(f, "division (/)"),
            BinaryOp::Mod => write!(f, "modulo (%)"),
            BinaryOp::Eq => write!(f, "equality (==)"),
            BinaryOp::Ne => write!(f, "inequality (!=)"),
            BinaryOp::Lt => write!(f, "less than (<)"),
            BinaryOp::Le => write!(f, "less than or equal (<=)"),
            BinaryOp::Gt => write!(f, "greater than (>)"),
            BinaryOp::Ge => write!(f, "greater than or equal (>=)"),
            BinaryOp::And => write!(f, "logical AND (&&)"),
            BinaryOp::Or => write!(f, "logical OR (||)"),
        }
    }
}

/// A type name for type casting.
///
/// Used in cast expressions to specify the target type.
///
/// # Mermaid Syntax
///
/// ```text
/// x as int
/// y as str
/// ```
///
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TypeName {
    /// The integer type (`int`).
    ///
    /// Represents 64-bit signed integers.
    Int,

    /// The string type (`str`).
    ///
    /// Represents UTF-8 strings.
    Str,
}
