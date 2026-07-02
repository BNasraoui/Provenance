mod enums;
mod ids;
mod records;

pub use enums::*;
pub use ids::*;
pub use records::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enriched_v1_records_roundtrip_without_schema_bump() {
        let source = serde_json::json!({
            "schema_version": 1,
            "scope_id": "default",
            "id": "source_sah",
            "name": "Support at Home",
            "source_type": "legislation",
            "url": "https://example.test/sah",
            "reference": "Department guidance",
            "originThread": "thread_req_origin",
            "originMessage": "msg_000001"
        });
        let requirement = serde_json::json!({
            "schema_version": 1,
            "scope_id": "default",
            "id": "req_sah",
            "statement": "Support at Home shall be traceable",
            "description": "Cloud import description",
            "status": "discovery",
            "source_refs": [{"source_id": "source_sah", "clause": "Program overview"}],
            "originThread": "thread_req_origin",
            "originMessage": "msg_000001"
        });
        let resolution = serde_json::json!({
            "schema_version": 1,
            "scope_id": "default",
            "id": "res_sah",
            "title": "SAH extraction",
            "position": "Keep as draft extraction",
            "rationale": "Needs human review",
            "status": "draft",
            "review_on": null,
            "review_triggers": [],
            "context": "Codebase scan",
            "enforcement": "specification",
            "confidence": 0.91,
            "originThread": "thread_req_origin",
            "originMessage": "msg_000001"
        });
        let rule = serde_json::json!({
            "schema_version": 1,
            "scope_id": "default",
            "id": "rule_sah_001",
            "rule_code": "SAH-001",
            "name": "SAH rule",
            "description": "Rule description",
            "statement": "Draft rule shall stay draft",
            "status": "draft",
            "severity": "high",
            "rule_type": "business",
            "modality": "obligation",
            "confidence": 0.98,
            "extraction_method": "manual",
            "source_document": "Example-API-main/src/example.php",
            "source_section": "lines 1-3",
            "expression": {},
            "inputs": [],
            "originThread": "thread_req_origin",
            "originMessage": "msg_000001"
        });

        let source: Source = serde_json::from_value(source).unwrap();
        let requirement: Requirement = serde_json::from_value(requirement).unwrap();
        let resolution: Resolution = serde_json::from_value(resolution).unwrap();
        let rule: Rule = serde_json::from_value(rule).unwrap();

        let source = serde_json::to_value(source).unwrap();
        let requirement = serde_json::to_value(requirement).unwrap();
        let resolution = serde_json::to_value(resolution).unwrap();
        let rule = serde_json::to_value(rule).unwrap();

        assert_eq!(source["schema_version"], 1);
        assert_eq!(source["source_type"], "legislation");
        assert_eq!(source["reference"], "Department guidance");
        assert_eq!(source["origin_thread"], "thread_req_origin");
        assert_eq!(source["origin_message"], "msg_000001");
        assert_eq!(requirement["schema_version"], 1);
        assert_eq!(requirement["status"], "discovery");
        assert_eq!(requirement["description"], "Cloud import description");
        assert_eq!(requirement["source_refs"][0]["clause"], "Program overview");
        assert_eq!(requirement["origin_thread"], "thread_req_origin");
        assert_eq!(requirement["origin_message"], "msg_000001");
        assert_eq!(resolution["schema_version"], 1);
        assert_eq!(resolution["status"], "draft");
        assert_eq!(resolution["confidence"], 0.91);
        assert_eq!(resolution["origin_thread"], "thread_req_origin");
        assert_eq!(resolution["origin_message"], "msg_000001");
        assert_eq!(rule["schema_version"], 1);
        assert_eq!(rule["status"], "draft");
        assert_eq!(rule["rule_type"], "business");
        assert_eq!(
            rule["source_document"],
            "Example-API-main/src/example.php"
        );
        assert_eq!(rule["origin_thread"], "thread_req_origin");
        assert_eq!(rule["origin_message"], "msg_000001");
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn ideation_output_records_roundtrip_without_schema_bump() {
        let contribution = serde_json::json!({
            "schema_version": 1,
            "scope_id": "default",
            "id": "contrib_reviewer_001",
            "target": {"artifact_type": "requirement", "artifact_id": "req_overtime"},
            "participant_slot": "reviewer",
            "stance": "support",
            "strongest_finding": "The requirement is supported by the award clause.",
            "evidence_references": [{
                "reference_id": "evidence_award_clause",
                "evidence_type": "source",
                "summary": "SCHADS overtime clause",
                "file_path": "src/payroll/overtime.rs",
                "line": 42
            }],
            "material_claims": [{
                "claim_id": "claim_overtime_threshold",
                "statement": "Overtime starts after the award threshold.",
                "evidence_type": "source",
                "evidence_reference_ids": ["evidence_award_clause"]
            }],
            "risks": ["Payroll underpayment if the threshold is wrong"],
            "objections": [],
            "challenges": [],
            "suggested_artifact_changes": [{
                "artifact_type": "requirement",
                "artifact_id": "req_overtime",
                "change_type": "update",
                "supporting_claim_ids": ["claim_overtime_threshold"],
                "summary": "Clarify the threshold source."
            }],
            "unsupported_recommendations": [{
                "recommendation": "Check enterprise agreement overrides.",
                "marker": "exploratory"
            }],
            "uncertainty": {"level": "medium", "rationale": "Agreement overrides were not reviewed."},
            "open_questions": ["Does the agreement override SCHADS?"]
        });
        let synthesis = serde_json::json!({
            "schema_version": 1,
            "scope_id": "default",
            "id": "synth_overtime_001",
            "target": {"artifact_type": "requirement", "artifact_id": "req_overtime"},
            "summary": "Participants agree the threshold needs explicit traceability.",
            "consensus": [{
                "statement": "The requirement needs an award source reference.",
                "supporting_participant_slots": ["reviewer"],
                "evidence_reference_ids": ["evidence_award_clause"]
            }],
            "contested_claims": [{
                "claim_id": "claim_agreement_override",
                "statement": "The agreement overrides the award.",
                "supporting_participant_slots": [],
                "opposing_participant_slots": ["reviewer"],
                "evidence_quality": "unsupported"
            }],
            "minority_objections": [{
                "participant_slot": "reviewer",
                "objection": "Agreement coverage still needs checking.",
                "evidence_reference_ids": []
            }],
            "evidence_gaps": [{
                "question": "Which agreement applies?",
                "needed_evidence_type": "source",
                "blocking_promotion": true
            }],
            "unsupported_speculation": [{
                "statement": "The agreement probably matches SCHADS.",
                "originating_participant_slots": ["reviewer"],
                "marker": "unsupported"
            }],
            "open_questions": ["Which agreement applies?"],
            "suggested_artifacts": [{
                "proposal_key": "req-overtime-traceability",
                "proposal_type": "requirement_candidate",
                "summary": "Clarify source traceability.",
                "origin_participant_slots": ["reviewer"]
            }],
            "required_human_decisions": [{
                "decision_key": "decide_agreement_scope",
                "prompt": "Confirm the governing agreement.",
                "blocks_promotion": true
            }]
        });
        let proposal = serde_json::json!({
            "schema_version": 1,
            "scope_id": "default",
            "id": "proposal_overtime_traceability",
            "proposal_key": "req-overtime-traceability",
            "proposal_type": "requirement_candidate",
            "title": "Clarify overtime traceability",
            "summary": "Add source-backed threshold language.",
            "traceability": {
                "target": {"artifact_type": "requirement", "artifact_id": "req_overtime"},
                "source_ids": ["source_schads"],
                "evidence_references": [{
                    "reference_id": "evidence_code_line",
                    "evidence_type": "artifact",
                    "summary": "Existing payroll check",
                    "file_path": "src/payroll/overtime.rs",
                    "line": 42
                }],
                "supporting_claim_ids": ["claim_overtime_threshold"]
            },
            "promotion_state": "proposed"
        });
        let decision = serde_json::json!({
            "schema_version": 1,
            "scope_id": "default",
            "id": "decision_overtime_traceability",
            "proposal_id": "proposal_overtime_traceability",
            "decision": "accepted",
            "rationale": "Human confirmed the source traceability.",
            "actor": {"identity_type": "human", "id": "ben", "name": "Ben"},
            "canonical_artifact": {"artifact_type": "requirement", "artifact_id": "req_overtime"}
        });

        let contribution: Contribution = serde_json::from_value(contribution).unwrap();
        let synthesis: SynthesisPacket = serde_json::from_value(synthesis).unwrap();
        let proposal: ProposalCard = serde_json::from_value(proposal).unwrap();
        let decision: PromotionDecisionRecord = serde_json::from_value(decision).unwrap();

        assert_eq!(contribution.schema_version, SchemaVersion(1));
        assert_eq!(
            contribution.evidence_references[0].file_path.as_deref(),
            Some("src/payroll/overtime.rs")
        );
        assert!(synthesis.evidence_gaps[0].blocking_promotion);
        assert_eq!(
            proposal.traceability.source_ids[0].as_str(),
            "source_schads"
        );
        assert_eq!(proposal.promotion_state, PromotionState::Proposed);
        assert_eq!(decision.decision, PromotionDecision::Accepted);

        assert_eq!(
            serde_json::to_value(contribution).unwrap()["schema_version"],
            1
        );
        assert_eq!(
            serde_json::to_value(synthesis).unwrap()["suggested_artifacts"][0]["proposal_type"],
            "requirement_candidate"
        );
        assert_eq!(
            serde_json::to_value(proposal).unwrap()["traceability"]["evidence_references"][0]
                ["line"],
            42
        );
        assert_eq!(
            serde_json::to_value(decision).unwrap()["actor"]["identity_type"],
            "human"
        );
    }

    #[test]
    fn proposal_and_promotion_decision_accept_convex_id_aliases() {
        let proposal = serde_json::json!({
            "schema_version": 1,
            "scope_id": "default",
            "proposalId": "proposal_overtime_traceability",
            "proposalKey": "req-overtime-traceability",
            "proposalType": "requirement_candidate",
            "title": "Clarify overtime traceability",
            "summary": "Add source-backed threshold language.",
            "traceability": {
                "target": {"artifactType": "requirement", "artifactId": "req_overtime"},
                "sourceIds": ["source_schads"],
                "evidenceReferences": [{
                    "referenceId": "evidence_code_line",
                    "evidenceType": "artifact",
                    "summary": "Existing payroll check",
                    "filePath": "src/payroll/overtime.rs",
                    "line": 42
                }],
                "supportingClaimIds": ["claim_overtime_threshold"]
            },
            "promotionState": "proposed"
        });
        let decision = serde_json::json!({
            "schema_version": 1,
            "scope_id": "default",
            "promotionDecisionId": "decision_overtime_traceability",
            "proposalId": "proposal_overtime_traceability",
            "decision": "accepted",
            "rationale": "Human confirmed the source traceability.",
            "decidedBy": {"identityType": "human", "userId": "ben", "name": "Ben"},
            "canonicalArtifact": {"artifactType": "requirement", "artifactId": "req_overtime"}
        });

        let proposal: ProposalCard = serde_json::from_value(proposal).unwrap();
        let decision: PromotionDecisionRecord = serde_json::from_value(decision).unwrap();

        assert_eq!(proposal.id.as_str(), "proposal_overtime_traceability");
        assert_eq!(proposal.proposal_type, ProposalType::RequirementCandidate);
        assert_eq!(decision.id.as_str(), "decision_overtime_traceability");
        assert_eq!(
            decision.proposal_id.as_str(),
            "proposal_overtime_traceability"
        );
        assert_eq!(decision.actor.id, "ben");
    }
}
