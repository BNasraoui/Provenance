use super::{read_edge_shards, read_jsonl, StateStore};
use crate::{jsonl::AdvisoryLock, shards};
use provenance_core::{
    Contribution, Edge, PromotionDecisionRecord, ProposalCard, Requirement, Resolution, Rule,
    ScopeId, Source,
};

/// An owned, immutable view of every record used by graph and evidence policy.
#[derive(Debug)]
pub struct ScopeSnapshot {
    pub scope: ScopeId,
    pub sources: Vec<Source>,
    pub requirements: Vec<Requirement>,
    pub resolutions: Vec<Resolution>,
    pub rules: Vec<Rule>,
    pub edges: Vec<Edge>,
    pub proposals: Vec<ProposalCard>,
    pub contributions: Vec<Contribution>,
    pub promotion_decisions: Vec<PromotionDecisionRecord>,
}

impl StateStore {
    /// Reads a scope while holding the state-generation lock exclusively.
    /// Writers hold a shared generation lock for their complete mutation.
    pub fn scope_snapshot(&self, scope: &ScopeId) -> anyhow::Result<ScopeSnapshot> {
        let _guard = AdvisoryLock::exclusive(&self.layout.state_snapshot_lock_path())?;
        Ok(ScopeSnapshot {
            scope: scope.clone(),
            sources: read_jsonl(&shards::sources_path(&self.layout, scope))?,
            requirements: read_jsonl(&shards::requirements_path(&self.layout, scope))?,
            resolutions: read_jsonl(&shards::resolutions_path(&self.layout, scope))?,
            rules: read_jsonl(&shards::rules_path(&self.layout, scope))?,
            edges: read_edge_shards(&self.layout)?
                .into_iter()
                .filter(|edge| edge.scope_id == *scope)
                .collect(),
            proposals: read_jsonl(&shards::proposal_cards_path(&self.layout, scope))?,
            contributions: read_jsonl(&shards::contributions_path(&self.layout, scope))?,
            promotion_decisions: read_jsonl(&shards::promotion_decisions_path(
                &self.layout,
                scope,
            ))?,
        })
    }
}
