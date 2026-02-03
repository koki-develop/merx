use std::collections::HashMap;

use crate::ast::{Edge, EdgeLabel, Node};

use super::error::ValidationError;

pub(super) fn insert_node(
    nodes: &mut HashMap<String, Node>,
    node: Node,
) -> Result<(), ValidationError> {
    let node_id = node.id().to_string();
    match nodes.get(&node_id) {
        Some(existing) => match (existing, &node) {
            // A bare Start/End reference (no label) does not conflict with any existing Start/End.
            (Node::Start { .. }, Node::Start { label: None })
            | (Node::End { .. }, Node::End { label: None }) => Ok(()),

            // A labeled Start/End upgrades an existing bare reference.
            (Node::Start { label: None }, Node::Start { label: Some(_) })
            | (Node::End { label: None }, Node::End { label: Some(_) }) => {
                nodes.insert(node_id, node);
                Ok(())
            }

            // Identical redefinition is allowed.
            (existing, new) if existing == new => Ok(()),

            _ => Err(ValidationError::new(format!(
                "Node '{}' is defined multiple times",
                node_id
            ))),
        },
        None => {
            nodes.insert(node_id, node);
            Ok(())
        }
    }
}

pub(super) fn validate_flowchart(
    nodes: &HashMap<String, Node>,
    edges: &[Edge],
) -> Result<(), ValidationError> {
    // Validate: condition nodes must have both Yes and No edges
    for node in nodes.values() {
        if let Node::Condition { id, .. } = node {
            let mut has_yes = false;
            let mut has_no = false;

            for edge in edges {
                if &edge.from != id {
                    continue;
                }

                match &edge.label {
                    Some(EdgeLabel::Yes) => {
                        if has_yes {
                            return Err(ValidationError::new(format!(
                                "Condition node '{}' has multiple 'Yes' edges",
                                id
                            )));
                        }
                        has_yes = true;
                    }
                    Some(EdgeLabel::No) => {
                        if has_no {
                            return Err(ValidationError::new(format!(
                                "Condition node '{}' has multiple 'No' edges",
                                id
                            )));
                        }
                        has_no = true;
                    }
                    Some(EdgeLabel::Custom(s)) => {
                        return Err(ValidationError::new(format!(
                            "Condition node '{}' must have 'Yes' or 'No' label, but got '{}'",
                            id, s
                        )));
                    }
                    None => {
                        return Err(ValidationError::new(format!(
                            "Edge from condition node '{}' must have 'Yes' or 'No' label",
                            id
                        )));
                    }
                }
            }

            if !has_yes {
                return Err(ValidationError::new(format!(
                    "Condition node '{}' is missing 'Yes' edge",
                    id
                )));
            }
            if !has_no {
                return Err(ValidationError::new(format!(
                    "Condition node '{}' is missing 'No' edge",
                    id
                )));
            }
        }
    }

    // Validate: Flowchart must have Start and End nodes
    if !nodes.values().any(|n| matches!(n, Node::Start { .. })) {
        return Err(ValidationError::new("Missing 'Start' node"));
    }
    if !nodes.values().any(|n| matches!(n, Node::End { .. })) {
        return Err(ValidationError::new("Missing 'End' node"));
    }

    // Validate: All edge references must point to defined nodes
    for edge in edges {
        if !nodes.contains_key(&edge.from) {
            return Err(ValidationError::new(format!(
                "Undefined node '{}' referenced in edge from '{}' to '{}'",
                edge.from, edge.from, edge.to
            )));
        }
        if !nodes.contains_key(&edge.to) {
            return Err(ValidationError::new(format!(
                "Undefined node '{}' referenced in edge from '{}' to '{}'",
                edge.to, edge.from, edge.to
            )));
        }
    }

    // Validate: End node must not have outgoing edges
    for edge in edges {
        if edge.from == "End" {
            return Err(ValidationError::new("End node cannot have outgoing edges"));
        }
    }

    // Validate: Non-condition nodes must have at most one outgoing edge
    let mut edge_counts: HashMap<String, usize> = HashMap::new();
    for edge in edges {
        *edge_counts.entry(edge.from.clone()).or_insert(0) += 1;
    }
    for (node_id, count) in edge_counts {
        if count > 1 {
            // Condition nodes are allowed to have 2 edges (Yes and No)
            let is_condition = nodes
                .get(&node_id)
                .is_some_and(|n| matches!(n, Node::Condition { .. }));
            if !is_condition {
                return Err(ValidationError::new(format!(
                    "Node '{}' has multiple outgoing edges (expected at most 1)",
                    node_id
                )));
            }
        }
    }

    // Validate: exit code is only allowed on edges pointing to the End node
    for edge in edges {
        if edge.exit_code.is_some() && edge.to != "End" {
            return Err(ValidationError::new(format!(
                "Exit code can only be specified on edges to 'End' node, but found on edge from '{}' to '{}'",
                edge.from, edge.to
            )));
        }
    }

    Ok(())
}
