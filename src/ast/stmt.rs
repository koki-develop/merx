use serde::Serialize;

use super::Expr;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Statement {
    Assign { variable: String, value: Expr },
    Print { expr: Expr },
    Error { message: Expr },
}
