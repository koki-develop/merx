//! Expression types for computations and values.
//!
//! Expressions form the computational core of the merx language. They can
//! represent literals, variables, operators, type casts, and user input.
//! Expressions are used in assignments, print statements, error statements,
//! and condition nodes.

use serde::Serialize;

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
/// # Serialization
///
/// Uses tagged enum serialization with `"type"` discriminator:
///
/// ```json
/// { "type": "int_lit", "value": 42 }
/// { "type": "binary", "op": "add", "left": {...}, "right": {...} }
/// ```
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
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
/// # Serialization
///
/// Serializes to lowercase: `"not"`, `"neg"`.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
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

/// A binary operator.
///
/// Binary operators take two operands and produce a result. Operators are
/// listed roughly in order of precedence (highest first within groups).
///
/// # Serialization
///
/// Serializes to lowercase: `"add"`, `"sub"`, `"eq"`, etc.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BinaryOp {
    /// Addition (`+`).
    ///
    /// Integer + Integer → Integer
    ///
    /// Note: String concatenation is NOT supported. To build strings,
    /// use explicit casting and multiple print statements.
    Add,

    /// Subtraction (`-`).
    ///
    /// Integer - Integer → Integer
    Sub,

    /// Multiplication (`*`).
    ///
    /// Integer * Integer → Integer
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
/// # Serialization
///
/// Serializes to lowercase: `"int"`, `"str"`.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
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
