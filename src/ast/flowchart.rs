use serde::Serialize;

use super::{Edge, Node};

#[derive(Debug, Clone, Serialize)]
pub struct Flowchart {
    pub direction: Direction,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Direction {
    Td,
    Tb,
    Lr,
    Rl,
    Bt,
}
