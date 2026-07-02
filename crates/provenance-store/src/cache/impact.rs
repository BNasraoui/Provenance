use crate::cache::serde_name;
use crate::layout::ProvenanceLayout;
use crate::state_store::StateStore;
use provenance_core::{Edge, EdgeType, NodeType, StableId};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ImpactDirection {
    Upstream,
    Downstream,
}

#[derive(Debug, serde::Serialize)]
pub struct ImpactNode {
    pub node_type: NodeType,
    pub id: String,
    pub hop_distance: u32,
    pub direction: ImpactDirection,
}

#[derive(Debug, serde::Serialize)]
pub struct ImpactView {
    pub origin_type: NodeType,
    pub origin_id: String,
    pub nodes: Vec<ImpactNode>,
    pub truncated: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct ImpactOptions {
    pub max_hops: u32,
    pub follow_indirect: bool,
}

pub fn analyze_impact(
    layout: &ProvenanceLayout,
    scope: &provenance_core::ScopeId,
    origin_type: NodeType,
    origin_id: &StableId,
    options: ImpactOptions,
) -> anyhow::Result<ImpactView> {
    let store = StateStore::new(layout.clone());
    let edges: Vec<_> = store
        .list_edges()?
        .into_iter()
        .filter(|edge| edge.scope_id == *scope)
        .filter(|edge| {
            options.follow_indirect
                || !matches!(
                    edge.edge_type,
                    EdgeType::DependsOn
                        | EdgeType::RefinesInto
                        | EdgeType::Contradicts
                        | EdgeType::Supersedes
                        | EdgeType::Spawns
                )
        })
        .collect();
    let mut nodes = BTreeMap::<(String, String, &'static str), ImpactNode>::new();
    let mut truncated = false;
    for direction in [ImpactDirection::Downstream, ImpactDirection::Upstream] {
        let mut seen =
            BTreeSet::from([(serde_name(&origin_type)?, origin_id.as_str().to_string())]);
        let mut queue = VecDeque::from([(origin_type, origin_id.clone(), 0_u32)]);
        while let Some((node_type, node_id, hops)) = queue.pop_front() {
            if hops >= options.max_hops {
                if edges
                    .iter()
                    .any(|edge| touches(edge, direction, node_type, &node_id))
                {
                    truncated = true;
                }
                continue;
            }
            for edge in edges
                .iter()
                .filter(|edge| touches(edge, direction, node_type, &node_id))
            {
                let (next_type, next_id) = match direction {
                    ImpactDirection::Downstream => (edge.to_type, edge.to_id.clone()),
                    ImpactDirection::Upstream => (edge.from_type, edge.from_id.clone()),
                };
                let key = (serde_name(&next_type)?, next_id.as_str().to_string());
                if seen.insert(key.clone()) {
                    let hop_distance = hops + 1;
                    nodes.insert(
                        (key.0, key.1, direction_key(direction)),
                        ImpactNode {
                            node_type: next_type,
                            id: next_id.as_str().to_string(),
                            hop_distance,
                            direction,
                        },
                    );
                    queue.push_back((next_type, next_id, hop_distance));
                }
            }
        }
    }
    Ok(ImpactView {
        origin_type,
        origin_id: origin_id.as_str().to_string(),
        nodes: nodes.into_values().collect(),
        truncated,
    })
}

fn touches(
    edge: &Edge,
    direction: ImpactDirection,
    node_type: NodeType,
    node_id: &StableId,
) -> bool {
    match direction {
        ImpactDirection::Downstream => edge.from_type == node_type && edge.from_id == *node_id,
        ImpactDirection::Upstream => edge.to_type == node_type && edge.to_id == *node_id,
    }
}

const fn direction_key(direction: ImpactDirection) -> &'static str {
    match direction {
        ImpactDirection::Upstream => "upstream",
        ImpactDirection::Downstream => "downstream",
    }
}
