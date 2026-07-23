use super::initialized_store;
use crate::state_store::{CreateProposalCardInput, ProposalDemand};
use provenance_core::{
    ArtifactLink, ArtifactLinkTargetType, IdeationEvidenceReference, IdeationEvidenceType,
    IdeationTarget, IdeationTargetType, PromotionState, ProposalTraceability, ProposalType,
    StableId, Topic, TopicStatus,
};

fn proposal_input(
    scope: &provenance_core::ScopeId,
    id: &str,
    target_type: IdeationTargetType,
    target_id: &str,
    path: Option<&str>,
    promotion_state: PromotionState,
) -> CreateProposalCardInput {
    CreateProposalCardInput {
        scope_id: scope.clone(),
        id: StableId::new(id).unwrap(),
        proposal_key: id.into(),
        proposal_type: ProposalType::RequirementCandidate,
        title: id.into(),
        summary: format!("Summary for {id}"),
        confidence: None,
        traceability: ProposalTraceability {
            target: IdeationTarget {
                artifact_type: target_type,
                artifact_id: StableId::new(target_id).unwrap(),
            },
            source_ids: Vec::new(),
            evidence_references: path
                .map(|path| IdeationEvidenceReference {
                    reference_id: StableId::new(format!("evidence_{id}")).unwrap(),
                    evidence_type: IdeationEvidenceType::Artifact,
                    summary: "Code evidence".into(),
                    file_path: Some(path.into()),
                    line: Some(42),
                })
                .into_iter()
                .collect(),
            supporting_claim_ids: Vec::new(),
        },
        builds_on: Vec::new(),
        promotion_state,
        duplicate_of: None,
        superseded_by: None,
    }
}

#[test]
fn changed_paths_surface_only_undisposed_proposals_with_matching_evidence_sites() {
    let (_dir, store, scope) = initialized_store();
    for input in [
        proposal_input(
            &scope,
            "proposal_matching",
            IdeationTargetType::Requirement,
            "req_overtime",
            Some("src/payroll.rs"),
            PromotionState::Proposed,
        ),
        proposal_input(
            &scope,
            "proposal_other_path",
            IdeationTargetType::Requirement,
            "req_overtime",
            Some("src/leave.rs"),
            PromotionState::Proposed,
        ),
    ] {
        store.create_proposal_card(input).unwrap();
    }

    let surfaced = store
        .surface_proposals(
            &scope,
            &ProposalDemand::for_changed_paths(["src/payroll.rs"]),
        )
        .unwrap();

    assert_eq!(surfaced.len(), 1);
    assert_eq!(surfaced[0].proposal.id.as_str(), "proposal_matching");
    assert_eq!(
        serde_json::to_value(&surfaced[0].reasons).unwrap(),
        serde_json::json!([{"trigger":"evidence_site","path":"src/payroll.rs"}])
    );
}

#[test]
fn asserted_proposal_still_surfaces_only_on_exact_demand_path() {
    let (_dir, store, scope) = initialized_store();
    store
        .create_proposal_card(proposal_input(
            &scope,
            "proposal_asserted",
            IdeationTargetType::Requirement,
            "req_overtime",
            Some("src/payroll.rs"),
            PromotionState::Proposed,
        ))
        .unwrap();
    crate::jsonl::write_jsonl_atomic(
        &crate::shards::assertion_records_path(&store.layout, &scope),
        &[provenance_core::AssertionRecord {
            schema_version: provenance_core::SchemaVersion(1),
            scope_id: scope.clone(),
            id: provenance_core::AssertionId::new("assertion_a").unwrap(),
            proposal_id: StableId::new("proposal_asserted").unwrap(),
            synthesis_packet_id: StableId::new("synthesis_a").unwrap(),
            supporting_claim_ids: Vec::new(),
        }],
    )
    .unwrap();

    assert!(store
        .surface_proposals(&scope, &ProposalDemand::for_changed_paths(["src/other.rs"]),)
        .unwrap()
        .is_empty());
    assert_eq!(
        store
            .surface_proposals(
                &scope,
                &ProposalDemand::for_changed_paths(["src/payroll.rs"]),
            )
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn a_topic_claim_surfaces_proposals_in_its_explicit_territory() {
    let (_dir, store, scope) = initialized_store();
    let topic = Topic {
        schema_version: provenance_core::SchemaVersion(1),
        scope_id: scope.clone(),
        id: StableId::new("topic_overtime").unwrap(),
        requirement_id: StableId::new("req_overtime").unwrap(),
        title: "Overtime".into(),
        status: TopicStatus::Open,
        claimed_by: Some("agent-one".into()),
        claimed_at: Some(1),
        links: vec![ArtifactLink {
            target_type: ArtifactLinkTargetType::Rule,
            target_id: StableId::new("rule_overtime").unwrap(),
        }],
    };
    for input in [
        proposal_input(
            &scope,
            "proposal_topic",
            IdeationTargetType::Topic,
            "topic_overtime",
            None,
            PromotionState::Proposed,
        ),
        proposal_input(
            &scope,
            "proposal_requirement",
            IdeationTargetType::Requirement,
            "req_overtime",
            None,
            PromotionState::Proposed,
        ),
        proposal_input(
            &scope,
            "proposal_link",
            IdeationTargetType::Rule,
            "rule_overtime",
            None,
            PromotionState::Proposed,
        ),
        proposal_input(
            &scope,
            "proposal_outside",
            IdeationTargetType::Requirement,
            "req_leave",
            None,
            PromotionState::Proposed,
        ),
    ] {
        store.create_proposal_card(input).unwrap();
    }

    let surfaced = store
        .surface_proposals(&scope, &ProposalDemand::for_topic(&topic))
        .unwrap();

    assert_eq!(
        surfaced
            .iter()
            .map(|item| item.proposal.id.as_str())
            .collect::<Vec<_>>(),
        vec!["proposal_link", "proposal_requirement", "proposal_topic"]
    );
}

#[test]
fn proposal_demand_must_name_a_real_trigger() {
    let (_dir, store, scope) = initialized_store();

    let error = store
        .surface_proposals(
            &scope,
            &ProposalDemand::for_changed_paths(Vec::<String>::new()),
        )
        .unwrap_err();

    assert!(error
        .to_string()
        .contains("at least one changed path or territory target"));
}
