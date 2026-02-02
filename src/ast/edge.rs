use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Edge {
    pub from: String,
    pub to: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<EdgeLabel>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum EdgeLabel {
    Yes,
    No,
    #[serde(untagged)]
    Custom(String),
}

impl EdgeLabel {
    pub fn is_yes_or_no(&self) -> bool {
        matches!(self, EdgeLabel::Yes | EdgeLabel::No)
    }
}
