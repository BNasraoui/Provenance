use super::{read_edge_shards, read_jsonl, StateStore};
use crate::{jsonl::AdvisoryLock, shards};
use provenance_core::{
    Contribution, Edge, PromotionDecisionRecord, ProposalCard, Requirement, Resolution, Rule,
    ScopeId, Source,
};
use serde::de::DeserializeOwned;

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
            sources: read_scoped(
                &shards::sources_path(&self.layout, scope),
                |record: &Source| record.scope_id == *scope,
            )?,
            requirements: read_scoped(
                &shards::requirements_path(&self.layout, scope),
                |record: &Requirement| record.scope_id == *scope,
            )?,
            resolutions: read_scoped(
                &shards::resolutions_path(&self.layout, scope),
                |record: &Resolution| record.scope_id == *scope,
            )?,
            rules: read_scoped(&shards::rules_path(&self.layout, scope), |record: &Rule| {
                record.scope_id == *scope
            })?,
            edges: read_edge_shards(&self.layout)?
                .into_iter()
                .filter(|edge| edge.scope_id == *scope)
                .collect(),
            proposals: read_scoped(
                &shards::proposal_cards_path(&self.layout, scope),
                |record: &ProposalCard| record.scope_id == *scope,
            )?,
            contributions: read_scoped(
                &shards::contributions_path(&self.layout, scope),
                |record: &Contribution| record.scope_id == *scope,
            )?,
            promotion_decisions: read_scoped(
                &shards::promotion_decisions_path(&self.layout, scope),
                |record: &PromotionDecisionRecord| record.scope_id == *scope,
            )?,
        })
    }
}

fn read_scoped<T: DeserializeOwned>(
    path: &camino::Utf8Path,
    belongs_to_scope: impl Fn(&T) -> bool,
) -> anyhow::Result<Vec<T>> {
    Ok(read_jsonl(path)?
        .into_iter()
        .filter(belongs_to_scope)
        .collect())
}
