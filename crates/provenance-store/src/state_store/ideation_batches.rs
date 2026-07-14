use std::collections::{BTreeMap, BTreeSet};

use super::{IdeationLandingBatch, StateStore};
use crate::shards;
use provenance_core::{
    validate_ideation_aggregate, validate_proposal_intrinsic, AssertionRecord, IdeationAggregate,
    ProposalCard, ScopeId,
};

impl StateStore {
    /// Validates existing and incoming ideation as one aggregate, then appends one atomic record.
    pub fn land_ideation_batch(
        &self,
        scope_id: &ScopeId,
        incoming: IdeationLandingBatch,
        replace: bool,
    ) -> anyhow::Result<()> {
        self.with_ideation_lock(scope_id, || {
            self.write_ideation_batch(scope_id, incoming, replace)
        })
    }

    fn write_ideation_batch(
        &self,
        scope_id: &ScopeId,
        mut incoming: IdeationLandingBatch,
        replace: bool,
    ) -> anyhow::Result<()> {
        ensure_batch_scope(scope_id, &incoming)?;
        self.validate_raw_ideation_history(scope_id)?;
        for proposal in &incoming.proposals {
            validate_proposal_intrinsic(proposal)?;
        }
        let path = shards::ideation_landings_path(&self.layout, scope_id);
        self.mutate_jsonl_records(&path, |landings: &mut Vec<IdeationLandingBatch>| {
            let mut contributions = self.list_contributions(scope_id)?;
            let mut synthesis_packets = self.list_synthesis_packets(scope_id)?;
            let mut proposals = self.list_proposal_records(scope_id)?;
            let mut assertions = self.list_assertion_records(scope_id)?;
            let mut dispositions = self.list_promotion_decisions(scope_id)?;

            let asserted_claims = assertions
                .iter()
                .flat_map(|assertion| &assertion.supporting_claim_ids)
                .collect::<BTreeSet<_>>();
            for incoming_record in &incoming.contributions {
                if let Some(existing) = contributions
                    .iter()
                    .find(|record| record.id == incoming_record.id)
                {
                    anyhow::ensure!(
                        !existing
                            .material_claims
                            .iter()
                            .any(|claim| asserted_claims.contains(&claim.claim_id)),
                        "contribution supplies durable assertion evidence and cannot be replaced"
                    );
                }
            }
            for incoming_record in &incoming.synthesis_packets {
                anyhow::ensure!(
                    !assertions.iter().any(|assertion| {
                        assertion.synthesis_packet_id == incoming_record.id
                            && synthesis_packets
                                .iter()
                                .any(|record| record.id == incoming_record.id)
                    }),
                    "synthesis packet adjudicates a durable assertion and cannot be replaced"
                );
            }

            merge_records(
                "contribution",
                &mut contributions,
                &incoming.contributions,
                replace,
                |record| record.id.as_str().to_owned(),
            )?;
            merge_records(
                "synthesis packet",
                &mut synthesis_packets,
                &incoming.synthesis_packets,
                replace,
                |record| record.id.as_str().to_owned(),
            )?;
            for proposal in &incoming.proposals {
                anyhow::ensure!(
                    !proposals.iter().any(|record| record.id == proposal.id),
                    "proposal {} already exists and is immutable",
                    proposal.id.as_str()
                );
            }
            merge_records(
                "proposal",
                &mut proposals,
                &incoming.proposals,
                replace,
                |record| record.id.as_str().to_owned(),
            )?;
            merge_records(
                "assertion",
                &mut assertions,
                &incoming.assertions,
                false,
                |record| record.id.as_str().to_owned(),
            )?;
            merge_records(
                "disposition",
                &mut dispositions,
                &incoming.dispositions,
                false,
                |record| record.id.as_str().to_owned(),
            )?;
            validate_ideation_aggregate(IdeationAggregate {
                contributions: &contributions,
                synthesis_packets: &synthesis_packets,
                proposals: &proposals,
                assertions: &assertions,
                dispositions: &dispositions,
            })?;
            incoming.proposals = topological_proposals(&incoming.proposals, &assertions)?;
            landings.push(incoming.clone());
            Ok(())
        })
    }
}

fn ensure_batch_scope(scope_id: &ScopeId, incoming: &IdeationLandingBatch) -> anyhow::Result<()> {
    for (kind, actual) in incoming
        .contributions
        .iter()
        .map(|record| ("contribution", &record.scope_id))
        .chain(
            incoming
                .synthesis_packets
                .iter()
                .map(|record| ("synthesis packet", &record.scope_id)),
        )
        .chain(
            incoming
                .proposals
                .iter()
                .map(|record| ("proposal", &record.scope_id)),
        )
        .chain(
            incoming
                .assertions
                .iter()
                .map(|record| ("assertion", &record.scope_id)),
        )
        .chain(
            incoming
                .dispositions
                .iter()
                .map(|record| ("disposition", &record.scope_id)),
        )
    {
        anyhow::ensure!(
            actual == scope_id,
            "{kind} scope_id must match landing scope"
        );
    }
    Ok(())
}

fn merge_records<T: Clone>(
    kind: &str,
    existing: &mut Vec<T>,
    incoming: &[T],
    replace: bool,
    id: impl Fn(&T) -> String,
) -> anyhow::Result<()> {
    let mut incoming_ids = BTreeSet::new();
    for record in incoming {
        let record_id = id(record);
        anyhow::ensure!(
            incoming_ids.insert(record_id.clone()),
            "duplicate {kind} id {record_id} in run"
        );
        if let Some(index) = existing.iter().position(|current| id(current) == record_id) {
            anyhow::ensure!(replace, "{kind} {record_id} already exists");
            existing[index] = record.clone();
        } else {
            existing.push(record.clone());
        }
    }
    Ok(())
}

fn topological_proposals(
    incoming: &[ProposalCard],
    assertions: &[AssertionRecord],
) -> anyhow::Result<Vec<ProposalCard>> {
    let assertion_owner = assertions
        .iter()
        .map(|assertion| (assertion.id.as_str(), assertion.proposal_id.as_str()))
        .collect::<BTreeMap<_, _>>();
    let incoming_ids = incoming
        .iter()
        .map(|proposal| proposal.id.as_str())
        .collect::<BTreeSet<_>>();
    let mut remaining = incoming.iter().collect::<Vec<_>>();
    remaining.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
    let mut emitted = BTreeSet::new();
    let mut ordered = Vec::new();
    while !remaining.is_empty() {
        let index = remaining.iter().position(|proposal| {
            proposal.builds_on.iter().all(|assertion_id| {
                assertion_owner
                    .get(assertion_id.as_str())
                    .is_none_or(|owner| !incoming_ids.contains(owner) || emitted.contains(owner))
            })
        });
        let index =
            index.ok_or_else(|| anyhow::anyhow!("proposal assertion lineage contains a cycle"))?;
        let proposal = remaining.remove(index);
        emitted.insert(proposal.id.as_str());
        ordered.push(proposal.clone());
    }
    Ok(ordered)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_store::tests::initialized_store;

    fn batch() -> IdeationLandingBatch {
        serde_json::from_value(serde_json::json!({
            "contributions": [{
                "schema_version": 1, "scope_id": "default", "id": "contribution_a",
                "target": {"artifact_type": "requirement", "artifact_id": "req_a"},
                "participant_slot": "extractor", "stance": "support", "strongest_finding": "Observed",
                "evidence_references": [{"reference_id": "evidence_a", "evidence_type": "source", "summary": "Pinned"}],
                "material_claims": [{"claim_id": "claim_a", "statement": "Observed", "evidence_type": "source", "evidence_reference_ids": ["evidence_a"]}],
                "risks": [], "objections": [], "challenges": [], "suggested_artifact_changes": [],
                "unsupported_recommendations": [], "uncertainty": {"level": "low", "rationale": "Direct"}, "open_questions": []
            }],
            "synthesis_packets": [{
                "schema_version": 1, "scope_id": "default", "id": "synthesis_a",
                "target": {"artifact_type": "requirement", "artifact_id": "req_a"}, "summary": "Adjudicated",
                "consensus": [], "contested_claims": [], "minority_objections": [], "evidence_gaps": [],
                "unsupported_speculation": [], "open_questions": [],
                "suggested_artifacts": [{"proposal_id": "proposal_parent", "proposal_key": "parent", "proposal_type": "requirement_candidate", "summary": "Parent", "origin_participant_slots": ["extractor"]}],
                "required_human_decisions": []
            }],
            "proposals": [{
                "schema_version": 1, "scope_id": "default", "id": "proposal_child", "proposal_key": "child",
                "proposal_type": "question", "title": "Child", "summary": "Child",
                "traceability": {"target": {"artifact_type": "requirement", "artifact_id": "req_a"}, "source_ids": [], "evidence_references": [], "supporting_claim_ids": []},
                "promotion_state": "proposed", "builds_on": ["assertion_parent"]
            }, {
                "schema_version": 1, "scope_id": "default", "id": "proposal_parent", "proposal_key": "parent",
                "proposal_type": "requirement_candidate", "title": "Parent", "summary": "Parent",
                "traceability": {"target": {"artifact_type": "requirement", "artifact_id": "req_a"}, "source_ids": [], "evidence_references": [], "supporting_claim_ids": ["claim_a"]},
                "promotion_state": "proposed"
            }],
            "assertions": [{
                "schema_version": 1, "scope_id": "default", "id": "assertion_parent",
                "proposal_id": "proposal_parent", "synthesis_packet_id": "synthesis_a",
                "supporting_claim_ids": ["claim_a"]
            }]
        })).unwrap()
    }

    #[test]
    fn child_before_parent_is_validated_and_landed_as_one_aggregate() {
        let (_dir, store, scope) = initialized_store();
        store.land_ideation_batch(&scope, batch(), false).unwrap();
        assert_eq!(store.list_proposal_cards(&scope).unwrap().len(), 2);
        assert_eq!(store.list_assertion_records(&scope).unwrap().len(), 1);
    }

    #[test]
    fn missing_lineage_rejects_the_whole_batch_without_partial_landing() {
        let (_dir, store, scope) = initialized_store();
        let mut incoming = batch();
        incoming.proposals[0].builds_on =
            vec![provenance_core::AssertionId::new("assertion_missing").unwrap()];
        let err = store
            .land_ideation_batch(&scope, incoming, false)
            .unwrap_err();
        assert!(err.to_string().contains("assertion_missing does not exist"));
        assert!(store.list_contributions(&scope).unwrap().is_empty());
        assert!(store.list_synthesis_packets(&scope).unwrap().is_empty());
        assert!(store.list_proposal_cards(&scope).unwrap().is_empty());
        assert!(store.list_assertion_records(&scope).unwrap().is_empty());
    }
}
