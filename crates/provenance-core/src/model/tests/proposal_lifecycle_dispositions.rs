#[test]
fn rejected_disposition_allows_blocked_or_contested_proposal_without_assertion() {
    for blocked in ["evidence_gap", "contested_claim"] {
        let (contribution, mut synthesis, proposal, _) = lifecycle_fixture();
        if blocked == "evidence_gap" {
            synthesis["evidence_gaps"] = serde_json::json!([{
                "question": "Unknown", "needed_evidence_type": "source", "blocking_promotion": true
            }]);
        } else {
            synthesis["contested_claims"] = serde_json::json!([{
                "claim_id": "claim_a", "statement": "Disputed", "supporting_participant_slots": [],
                "opposing_participant_slots": ["refuter"], "evidence_quality": "weak"
            }]);
        }
        let contributions = vec![serde_json::from_value(contribution).unwrap()];
        let synthesis_packets = vec![serde_json::from_value(synthesis).unwrap()];
        let proposals = vec![serde_json::from_value(proposal).unwrap()];
        let dispositions = vec![serde_json::from_value(serde_json::json!({
            "schema_version": 1, "scope_id": "default", "id": format!("disposition_{blocked}"),
            "proposal_id": "proposal_a", "decision": "rejected", "rationale": "Did not pass adjudication",
            "actor": {"identity_type": "human", "id": "reviewer"}
        })).unwrap()];

        crate::validate_ideation_aggregate(crate::IdeationAggregate {
            legacy_policy: crate::LegacyProposalPolicy::ModernOnly,
            disposition_actor_ids: &["reviewer".into()],
            contributions: &contributions,
            synthesis_packets: &synthesis_packets,
            proposals: &proposals,
            assertions: &[],
            dispositions: &dispositions,
        })
        .unwrap();
    }
}

#[test]
fn accepted_disposition_still_requires_assertion() {
    let (contribution, synthesis, proposal, _) = lifecycle_fixture();
    let contributions = vec![serde_json::from_value(contribution).unwrap()];
    let synthesis_packets = vec![serde_json::from_value(synthesis).unwrap()];
    let proposals = vec![serde_json::from_value(proposal).unwrap()];
    let dispositions = vec![serde_json::from_value(serde_json::json!({
        "schema_version": 1, "scope_id": "default", "id": "disposition_a",
        "proposal_id": "proposal_a", "decision": "accepted", "rationale": "Reviewed",
        "actor": {"identity_type": "human", "id": "reviewer"}
    })).unwrap()];

    let error = crate::validate_ideation_aggregate(crate::IdeationAggregate {
        legacy_policy: crate::LegacyProposalPolicy::ModernOnly,
        disposition_actor_ids: &["reviewer".into()],
        contributions: &contributions,
        synthesis_packets: &synthesis_packets,
        proposals: &proposals,
        assertions: &[],
        dispositions: &dispositions,
    })
    .unwrap_err()
    .to_string();
    assert!(error.contains("must be asserted"), "{error}");
}

#[test]
fn rejected_or_deferred_disposition_can_override_asserted_state() {
    for decision in ["rejected", "deferred"] {
        let (contribution, synthesis, proposal, assertion) = lifecycle_fixture();
        let contributions = vec![serde_json::from_value(contribution).unwrap()];
        let synthesis_packets = vec![serde_json::from_value(synthesis).unwrap()];
        let proposals = vec![serde_json::from_value(proposal).unwrap()];
        let assertions = vec![serde_json::from_value(assertion).unwrap()];
        let dispositions = vec![serde_json::from_value(serde_json::json!({
            "schema_version": 1, "scope_id": "default", "id": format!("disposition_{decision}"),
            "proposal_id": "proposal_a", "decision": decision, "rationale": "Reviewed",
            "actor": {"identity_type": "human", "id": "reviewer"}
        })).unwrap()];

        crate::validate_ideation_aggregate(crate::IdeationAggregate {
            legacy_policy: crate::LegacyProposalPolicy::ModernOnly,
            disposition_actor_ids: &["reviewer".into()],
            contributions: &contributions,
            synthesis_packets: &synthesis_packets,
            proposals: &proposals,
            assertions: &assertions,
            dispositions: &dispositions,
        })
        .unwrap();
    }
}

#[test]
fn disposition_external_action_is_closed_and_round_trips_generically() {
    let value = serde_json::json!({
        "schema_version": 1,
        "scope_id": "default",
        "id": "disposition_a",
        "proposal_id": "proposal_a",
        "decision": "rejected",
        "rationale": "Implemented elsewhere",
        "actor": {"identity_type": "human", "id": "reviewer"},
        "external_action": {
            "system": "github",
            "scope": "acme/payroll",
            "kind": "commit",
            "key": "abc123"
        }
    });
    let disposition: crate::DispositionRecord = serde_json::from_value(value.clone()).unwrap();

    assert_eq!(serde_json::to_value(disposition).unwrap(), value);

    let mut unknown = value;
    unknown["external_action"]["workflow_state"] = serde_json::json!("deployed");
    let error = serde_json::from_value::<crate::DispositionRecord>(unknown)
        .unwrap_err()
        .to_string();
    assert!(error.contains("unknown field"), "{error}");
}

#[test]
fn disposition_external_action_rejects_null_and_blank_identity_parts() {
    let base = serde_json::json!({
        "schema_version": 1, "scope_id": "default", "id": "disposition_a",
        "proposal_id": "proposal_a", "decision": "rejected", "rationale": "Reviewed",
        "actor": {"identity_type": "human", "id": "reviewer"}
    });
    let mut null = base.clone();
    null["external_action"] = serde_json::Value::Null;
    assert!(serde_json::from_value::<crate::DispositionRecord>(null).is_err());

    for field in ["system", "scope", "kind", "key"] {
        let mut value = base.clone();
        value["external_action"] = serde_json::json!({
            "system": "github", "scope": "acme/payroll", "kind": "issue", "key": "44"
        });
        value["external_action"][field] = serde_json::json!("  ");
        let disposition: crate::DispositionRecord = serde_json::from_value(value).unwrap();
        let error = crate::validate_disposition_intrinsic(&disposition)
            .unwrap_err()
            .to_string();
        assert!(error.contains(field), "{error}");
    }
}
