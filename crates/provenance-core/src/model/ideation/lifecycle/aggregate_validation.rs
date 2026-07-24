use super::{
    assertion_validation, legacy_validation, lineage_validation, validate_proposal_intrinsic,
    IdeationAggregate,
};
use crate::model::{
    validate_disposition_intrinsic, DispositionRecord, PromotionState, ProposalCard,
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn validate(aggregate: IdeationAggregate<'_>) -> anyhow::Result<()> {
    validate_source_schema_versions(&aggregate)?;
    ensure_immutable_ids(
        "contribution",
        aggregate
            .contributions
            .iter()
            .map(|record| (record.id.as_str(), record)),
    )?;
    ensure_immutable_ids(
        "synthesis packet",
        aggregate
            .synthesis_packets
            .iter()
            .map(|record| (record.id.as_str(), record)),
    )?;
    ensure_immutable_ids(
        "proposal",
        aggregate
            .proposals
            .iter()
            .map(|record| (record.id.as_str(), record)),
    )?;
    legacy_validation::validate_records(
        aggregate.proposals,
        aggregate.dispositions,
        aggregate.legacy_policy,
    )?;
    validate_proposals(aggregate.proposals)?;
    ensure_immutable_ids(
        "assertion",
        aggregate
            .assertions
            .iter()
            .map(|record| (record.id.as_str(), record)),
    )?;
    ensure_immutable_ids(
        "disposition",
        aggregate
            .dispositions
            .iter()
            .map(|record| (record.id.as_str(), record)),
    )?;
    validate_disposition_intrinsics(aggregate.dispositions)?;

    let proposals = aggregate
        .proposals
        .iter()
        .map(|proposal| (proposal.id.as_str(), proposal))
        .collect::<BTreeMap<_, _>>();
    validate_actor_allowlist(&aggregate, &proposals)?;
    lineage_validation::validate(aggregate.proposals, aggregate.assertions)?;
    let synthesis_packets = aggregate
        .synthesis_packets
        .iter()
        .map(|packet| (packet.id.as_str(), packet))
        .collect::<BTreeMap<_, _>>();
    assertion_validation::validate_assertions(&aggregate, &proposals, &synthesis_packets)?;
    validate_disposition_links(&aggregate, &proposals)?;
    assertion_validation::ensure_qualifying_assertions(
        aggregate.proposals,
        aggregate.synthesis_packets,
        aggregate.assertions,
    )
}

fn validate_source_schema_versions(aggregate: &IdeationAggregate<'_>) -> anyhow::Result<()> {
    for contribution in aggregate.contributions {
        anyhow::ensure!(
            contribution.schema_version.0 == 1,
            "contribution schema_version must be 1"
        );
    }
    for packet in aggregate.synthesis_packets {
        anyhow::ensure!(
            packet.schema_version.0 == 1,
            "synthesis schema_version must be 1"
        );
    }
    Ok(())
}

fn validate_proposals(proposals: &[ProposalCard]) -> anyhow::Result<()> {
    for proposal in proposals {
        anyhow::ensure!(
            proposal.schema_version.0 == 1,
            "proposal schema_version must be 1"
        );
        if proposal.promotion_state == PromotionState::Proposed {
            validate_proposal_intrinsic(proposal)?;
        }
    }
    Ok(())
}

fn validate_disposition_intrinsics(dispositions: &[DispositionRecord]) -> anyhow::Result<()> {
    for disposition in dispositions {
        validate_disposition_intrinsic(disposition)?;
    }
    Ok(())
}

fn validate_actor_allowlist(
    aggregate: &IdeationAggregate<'_>,
    proposals: &BTreeMap<&str, &ProposalCard>,
) -> anyhow::Result<()> {
    for disposition in aggregate.dispositions {
        if proposals
            .get(disposition.proposal_id.as_str())
            .is_some_and(|proposal| proposal.promotion_state == PromotionState::Proposed)
        {
            anyhow::ensure!(
                aggregate
                    .disposition_actor_ids
                    .iter()
                    .any(|id| id == &disposition.actor.id),
                "disposition actor is not in the repository allowlist"
            );
        }
    }
    Ok(())
}

fn validate_disposition_links(
    aggregate: &IdeationAggregate<'_>,
    proposals: &BTreeMap<&str, &ProposalCard>,
) -> anyhow::Result<()> {
    let mut disposed = BTreeSet::new();
    for disposition in aggregate.dispositions {
        anyhow::ensure!(
            disposition.schema_version.0 == 1,
            "disposition schema_version must be 1"
        );
        let proposal = proposals
            .get(disposition.proposal_id.as_str())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "disposition proposal {} does not exist",
                    disposition.proposal_id.as_str()
                )
            })?;
        anyhow::ensure!(
            disposition.scope_id == proposal.scope_id,
            "disposition must share the proposal scope"
        );
        anyhow::ensure!(
            disposed.insert(disposition.proposal_id.as_str()),
            "proposal {} has multiple dispositions",
            disposition.proposal_id.as_str()
        );
        if proposal.promotion_state != PromotionState::Proposed {
            continue;
        }
        anyhow::ensure!(
            disposition.decision != super::super::DispositionDecision::Accepted
                || aggregate
                    .assertions
                    .iter()
                    .any(|assertion| assertion.proposal_id == proposal.id),
            "accepted proposal {} must be asserted before disposition",
            proposal.id.as_str()
        );
    }
    Ok(())
}

fn ensure_immutable_ids<'a, T: Serialize + 'a>(
    kind: &str,
    records: impl IntoIterator<Item = (&'a str, &'a T)>,
) -> anyhow::Result<()> {
    let mut seen = BTreeMap::new();
    for (id, record) in records {
        let value = serde_json::to_value(record)?;
        anyhow::ensure!(
            seen.insert(id, value).is_none(),
            "duplicate immutable {kind} id {id}"
        );
    }
    Ok(())
}
