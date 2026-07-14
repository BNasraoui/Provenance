use crate::cache::{find_gaps, GapItem};
use crate::layout::ProvenanceLayout;
use crate::state_store::StateStore;
use provenance_core::{
    Edge, EdgeType, Message, PromotionState, ProposalView, Requirement, Rule, Source, Thread,
};
use std::collections::BTreeSet;
use std::fmt::Write;

#[derive(Debug, serde::Serialize)]
pub struct RequirementGraphView {
    pub requirement: Requirement,
    pub sources: Vec<Source>,
    pub edges: Vec<Edge>,
}

#[derive(Debug, serde::Serialize)]
pub struct PrimeThreadView {
    pub thread: Thread,
    pub messages: Vec<Message>,
}

#[derive(Debug, serde::Serialize)]
pub struct PrimeContextView {
    pub scope_id: String,
    pub rules: Vec<Rule>,
    pub provisional_proposals: Vec<ProposalView>,
    pub gaps: Vec<GapItem>,
    pub threads: Vec<PrimeThreadView>,
}

pub fn prime_context(
    layout: &ProvenanceLayout,
    scope: &provenance_core::ScopeId,
    include_threads: bool,
) -> anyhow::Result<PrimeContextView> {
    let store = StateStore::new(layout.clone());
    let threads = if include_threads {
        let messages = store.list_messages(scope)?;
        store
            .list_threads(scope)?
            .into_iter()
            .filter(|thread| thread.status == provenance_core::ThreadStatus::Active)
            .map(|thread| PrimeThreadView {
                messages: messages
                    .iter()
                    .filter(|message| message.thread_id == thread.id)
                    .cloned()
                    .collect(),
                thread,
            })
            .collect()
    } else {
        Vec::new()
    };
    let provisional_proposals = store
        .list_proposal_cards(scope)?
        .into_iter()
        .filter(|proposal| {
            matches!(
                proposal.promotion_state,
                PromotionState::Proposed | PromotionState::Asserted
            )
        })
        .collect();
    Ok(PrimeContextView {
        scope_id: scope.as_str().to_string(),
        rules: store.list_rules(scope)?,
        provisional_proposals,
        gaps: find_gaps(layout, scope)?,
        threads,
    })
}

pub fn render_prime_markdown(view: &PrimeContextView) -> String {
    let mut out = format!(
        "# Provenance Prime\n\nScope: {}\n\n## Rules\n",
        view.scope_id
    );
    for rule in &view.rules {
        let _ = writeln!(out, "- {} ({})", rule.id.as_str(), rule.rule_code);
    }
    out.push_str("\n## Provisional proposals\n");
    if view.provisional_proposals.is_empty() {
        out.push_str("- none\n");
    }
    for proposal in &view.provisional_proposals {
        let status = match proposal.promotion_state {
            PromotionState::Asserted => "asserted; not human-ratified",
            PromotionState::Proposed => "proposed; not asserted",
            _ => unreachable!("prime filters final proposal states"),
        };
        let _ = writeln!(
            out,
            "- {} [{}]: {} — {}",
            proposal.id.as_str(),
            status,
            proposal.title,
            proposal.summary
        );
        if !proposal.builds_on.is_empty() {
            let lineage = proposal
                .builds_on
                .iter()
                .map(provenance_core::AssertionId::as_str)
                .collect::<Vec<_>>()
                .join(", ");
            let _ = writeln!(out, "  - builds on provisionally: {lineage}");
        }
    }
    out.push_str("\n## Gaps\n");
    if view.gaps.is_empty() {
        out.push_str("- none\n");
    }
    for gap in &view.gaps {
        let _ = writeln!(out, "- {}: {}", gap.subject(), gap.reason);
    }
    out.push_str("\n## Threads\n");
    for item in &view.threads {
        let _ = writeln!(
            out,
            "- {} on {}",
            item.thread.id.as_str(),
            item.thread.parent.node_id.as_str()
        );
        for message in &item.messages {
            let _ = writeln!(out, "  - {}: {}", message.id.as_str(), message.body);
        }
    }
    out
}

pub fn get_requirement_graph(
    layout: &ProvenanceLayout,
    scope: &provenance_core::ScopeId,
    requirement_id: &provenance_core::StableId,
) -> anyhow::Result<RequirementGraphView> {
    let store = StateStore::new(layout.clone());
    let requirement = store
        .list_requirements(scope)?
        .into_iter()
        .find(|requirement| requirement.id == *requirement_id)
        .ok_or_else(|| anyhow::anyhow!("requirement not found"))?;
    let edges: Vec<_> = store
        .list_edges()?
        .into_iter()
        .filter(|edge| {
            edge.scope_id == *scope
                && (edge.to_id == *requirement_id || edge.from_id == *requirement_id)
        })
        .collect();
    let source_ids: BTreeSet<_> = edges
        .iter()
        .filter(|edge| edge.edge_type == EdgeType::References && edge.to_id == *requirement_id)
        .map(|edge| edge.from_id.as_str().to_string())
        .collect();
    let sources = store
        .list_sources(scope)?
        .into_iter()
        .filter(|source| source_ids.contains(source.id.as_str()))
        .collect();
    Ok(RequirementGraphView {
        requirement,
        sources,
        edges,
    })
}
