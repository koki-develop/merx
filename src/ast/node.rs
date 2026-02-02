use serde::Serialize;

use super::{Expr, Statement};

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Node {
    Start,
    End,
    Process {
        id: String,
        statements: Vec<Statement>,
    },
    Condition {
        id: String,
        condition: Expr,
    },
}

impl Node {
    pub fn id(&self) -> &str {
        match self {
            Node::Start => "Start",
            Node::End => "End",
            Node::Process { id, .. } => id,
            Node::Condition { id, .. } => id,
        }
    }
}
