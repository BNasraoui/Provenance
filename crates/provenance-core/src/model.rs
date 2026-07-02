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
    fn enriched_source_and_requirement_records_roundtrip_without_schema_bump() {
        let source = serde_json::json!({
            "schema_version": 1,
            "scope_id": "default",
            "id": "source_sah",
            "name": "Support at Home",
            "source_type": "legislation",
            "url": "https://example.test/sah",
            "reference": "Department guidance",
            "commitPin": "5e1f2a9c4b6d8e0f1234567890abcdef12345678",
            "effectiveDate": 1_714_521_600_000_i64,
            "reviewDate": 1_717_200_000_000_i64,
            "supersededBy": "source_sah_2025",
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

        let source: Source = serde_json::from_value(source).unwrap();
        let requirement: Requirement = serde_json::from_value(requirement).unwrap();

        let source = serde_json::to_value(source).unwrap();
        let requirement = serde_json::to_value(requirement).unwrap();

        assert_eq!(source["schema_version"], 1);
        assert_eq!(source["source_type"], "legislation");
        assert_eq!(source["reference"], "Department guidance");
        assert_eq!(
            source["commit_pin"],
            "5e1f2a9c4b6d8e0f1234567890abcdef12345678"
        );
        assert_eq!(source["effective_date"], 1_714_521_600_000_i64);
        assert_eq!(source["review_date"], 1_717_200_000_000_i64);
        assert_eq!(source["superseded_by"], "source_sah_2025");
        assert_eq!(source["origin_thread"], "thread_req_origin");
        assert_eq!(source["origin_message"], "msg_000001");
        assert_eq!(requirement["schema_version"], 1);
        assert_eq!(requirement["status"], "discovery");
        assert_eq!(requirement["description"], "Cloud import description");
        assert_eq!(requirement["source_refs"][0]["clause"], "Program overview");
        assert_eq!(requirement["origin_thread"], "thread_req_origin");
        assert_eq!(requirement["origin_message"], "msg_000001");
    }

    #[test]
    fn source_commit_pin_must_be_hex_git_commit() {
        let source = serde_json::json!({
            "schema_version": 1,
            "scope_id": "default",
            "id": "source_codebase",
            "name": "Codebase",
            "source_type": "project_artifact",
            "commitPin": "main"
        });

        assert!(serde_json::from_value::<Source>(source)
            .unwrap_err()
            .to_string()
            .contains("commit pin"));
    }

    #[test]
    fn enriched_resolution_and_rule_records_roundtrip_without_schema_bump() {
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
            "inputs": [{
                "inputType": "regulatory",
                "reference": "SAH program manual",
                "summary": "Program rules reviewed"
            }],
            "madeBy": "Analyst One",
            "approvedBy": "Approver Two",
            "approvedAt": 1_714_780_800_000_i64,
            "supersededBy": "res_sah_2025",
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

        let resolution: Resolution = serde_json::from_value(resolution).unwrap();
        let rule: Rule = serde_json::from_value(rule).unwrap();

        let resolution = serde_json::to_value(resolution).unwrap();
        let rule = serde_json::to_value(rule).unwrap();

        assert_eq!(resolution["schema_version"], 1);
        assert_eq!(resolution["status"], "draft");
        assert_eq!(resolution["confidence"], 0.91);
        assert_eq!(resolution["inputs"][0]["input_type"], "regulatory");
        assert_eq!(resolution["inputs"][0]["reference"], "SAH program manual");
        assert_eq!(resolution["inputs"][0]["summary"], "Program rules reviewed");
        assert_eq!(resolution["made_by"], "Analyst One");
        assert_eq!(resolution["approved_by"], "Approver Two");
        assert_eq!(resolution["approved_at"], 1_714_780_800_000_i64);
        assert_eq!(resolution["superseded_by"], "res_sah_2025");
        assert_eq!(resolution["origin_thread"], "thread_req_origin");
        assert_eq!(resolution["origin_message"], "msg_000001");
        assert_eq!(rule["schema_version"], 1);
        assert_eq!(rule["status"], "draft");
        assert_eq!(rule["rule_type"], "business");
        assert_eq!(rule["source_document"], "Example-API-main/src/example.php");
        assert_eq!(rule["origin_thread"], "thread_req_origin");
        assert_eq!(rule["origin_message"], "msg_000001");
    }

    #[test]
    fn resolved_threads_roundtrip_as_v1_state() {
        let thread = serde_json::json!({
            "schema_version": 1,
            "scope_id": "default",
            "id": "thread_rule_rule_sah_001",
            "parent": {
                "node_type": "rule",
                "node_id": "rule_sah_001"
            },
            "status": "resolved",
            "created_at": 1
        });

        let thread: Thread = serde_json::from_value(thread).unwrap();
        let thread = serde_json::to_value(thread).unwrap();

        assert_eq!(thread["status"], "resolved");
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
                "evidence_reference_ids": ["evidence_award_clause"],
                "confidence": 0.87
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
            "confidence": 0.83,
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
            serde_json::to_value(&contribution).unwrap()["schema_version"],
            1
        );
        assert_eq!(
            serde_json::to_value(&contribution).unwrap()["material_claims"][0]["confidence"],
            0.87
        );
        assert_eq!(
            serde_json::to_value(&synthesis).unwrap()["suggested_artifacts"][0]["proposal_type"],
            "requirement_candidate"
        );
        assert_eq!(serde_json::to_value(&proposal).unwrap()["confidence"], 0.83);
        assert_eq!(
            serde_json::to_value(&proposal).unwrap()["traceability"]["evidence_references"][0]
                ["line"],
            42
        );
        assert_eq!(
            serde_json::to_value(&decision).unwrap()["actor"]["identity_type"],
            "human"
        );
    }

    #[test]
    fn confidence_scores_must_be_in_unit_interval() {
        let claim = serde_json::json!({
            "claim_id": "claim_overtime_threshold",
            "statement": "Overtime starts after the award threshold.",
            "evidence_type": "source",
            "evidence_reference_ids": ["evidence_award_clause"],
            "confidence": 1.01
        });
        let proposal = serde_json::json!({
            "schema_version": 1,
            "scope_id": "default",
            "id": "proposal_overtime_traceability",
            "proposal_key": "req-overtime-traceability",
            "proposal_type": "requirement_candidate",
            "title": "Clarify overtime traceability",
            "summary": "Add source-backed threshold language.",
            "confidence": -0.01,
            "traceability": {
                "target": {"artifact_type": "requirement", "artifact_id": "req_overtime"},
                "source_ids": ["source_schads"],
                "evidence_references": [],
                "supporting_claim_ids": ["claim_overtime_threshold"]
            },
            "promotion_state": "proposed"
        });

        assert!(serde_json::from_value::<MaterialClaim>(claim)
            .unwrap_err()
            .to_string()
            .contains("confidence"));
        assert!(serde_json::from_value::<ProposalCard>(proposal)
            .unwrap_err()
            .to_string()
            .contains("confidence"));
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

    #[test]
    fn shaping_records_roundtrip_from_convex_style_json() {
        let boundary = serde_json::json!({
            "schema_version": 1,
            "scope_id": "default",
            "id": "boundary_no_manual_payroll",
            "requirementId": "req_payroll",
            "statement": "Do not require manual payroll reconciliation",
            "sourceRef": {"sourceId": "source_schads", "clause": "28.1"}
        });
        let topic = serde_json::json!({
            "schema_version": 1,
            "scope_id": "default",
            "id": "topic_overtime",
            "requirementId": "req_payroll",
            "title": "Overtime eligibility",
            "status": "explored",
            "links": [{"targetType": "source", "targetId": "source_schads"}]
        });
        let question = serde_json::json!({
            "schema_version": 1,
            "scope_id": "default",
            "id": "question_overtime_threshold",
            "topicId": "topic_overtime",
            "requirementId": "req_payroll",
            "question": "Which threshold applies?",
            "resolutionMethod": "research",
            "status": "answered",
            "claimedBy": "agent-shaper",
            "claimedAt": 1_714_780_800_000_i64,
            "answer": "Use the SCHADS overtime threshold.",
            "links": [{"targetType": "resolution", "targetId": "res_overtime"}],
            "resolutionId": "res_overtime"
        });

        let boundary: Boundary = serde_json::from_value(boundary).unwrap();
        let topic: Topic = serde_json::from_value(topic).unwrap();
        let question: Question = serde_json::from_value(question).unwrap();

        assert_eq!(boundary.requirement_id.as_str(), "req_payroll");
        assert_eq!(
            boundary.source_ref.as_ref().unwrap().source_id.as_str(),
            "source_schads"
        );
        assert_eq!(topic.status, TopicStatus::Explored);
        assert_eq!(topic.links[0].target_type, ArtifactLinkTargetType::Source);
        assert_eq!(topic.claimed_by, None);
        assert_eq!(question.topic_id.as_str(), "topic_overtime");
        assert_eq!(question.resolution_method, ResolutionMethod::Research);
        assert_eq!(question.claimed_by.as_deref(), Some("agent-shaper"));
        assert_eq!(question.claimed_at, Some(1_714_780_800_000));
        assert_eq!(question.status, QuestionStatus::Answered);
        assert_eq!(
            question.resolution_id.as_ref().unwrap().as_str(),
            "res_overtime"
        );

        let boundary = serde_json::to_value(boundary).unwrap();
        let topic = serde_json::to_value(topic).unwrap();
        let question = serde_json::to_value(question).unwrap();

        assert_eq!(boundary["schema_version"], 1);
        assert_eq!(boundary["requirement_id"], "req_payroll");
        assert_eq!(boundary["source_ref"]["source_id"], "source_schads");
        assert_eq!(topic["status"], "explored");
        assert_eq!(topic["links"][0]["target_type"], "source");
        assert!(topic.get("claimed_by").is_none());
        assert!(topic.get("claimed_at").is_none());
        assert_eq!(question["resolution_method"], "research");
        assert_eq!(question["status"], "answered");
        assert_eq!(question["claimed_by"], "agent-shaper");
        assert_eq!(question["claimed_at"], 1_714_780_800_000_i64);
        assert_eq!(question["resolution_id"], "res_overtime");
    }

    #[test]
    fn requirement_fog_roundtrips_as_unstructured_text() {
        let requirement = serde_json::json!({
            "schema_version": 1,
            "scope_id": "default",
            "id": "req_share_links",
            "statement": "Provenance docs shareable via short-lived link",
            "status": "discovery",
            "fog": "access auditing; expiry configuration; something about revocation"
        });

        let requirement: Requirement = serde_json::from_value(requirement).unwrap();
        assert_eq!(
            requirement.fog.as_deref(),
            Some("access auditing; expiry configuration; something about revocation")
        );

        let requirement = serde_json::to_value(requirement).unwrap();
        assert_eq!(
            requirement["fog"],
            "access auditing; expiry configuration; something about revocation"
        );

        let without_fog = serde_json::json!({
            "schema_version": 1,
            "scope_id": "default",
            "id": "req_plain",
            "statement": "Plain requirement",
            "status": "active"
        });
        let without_fog: Requirement = serde_json::from_value(without_fog).unwrap();
        assert_eq!(without_fog.fog, None);
        assert!(serde_json::to_value(without_fog)
            .unwrap()
            .get("fog")
            .is_none());
    }

    #[test]
    fn topic_and_question_are_thread_parent_node_types_but_not_edge_endpoints() {
        assert_eq!(NodeType::parse("topic").unwrap(), NodeType::Topic);
        assert_eq!(NodeType::parse("question").unwrap(), NodeType::Question);
        assert!(crate::edge_validation::validate_edge_endpoint(
            EdgeType::DependsOn,
            NodeType::Topic,
            NodeType::Topic,
        )
        .is_err());
    }

    #[test]
    fn domain_service_records_roundtrip_without_hosted_fields() {
        let domain = serde_json::json!({
            "schema_version": 1,
            "scope_id": "default",
            "id": "domain_payroll",
            "name": "Payroll",
            "description": "Payroll compliance requirements",
            "color": "#3b82f6"
        });
        let requirement = serde_json::json!({
            "schema_version": 1,
            "scope_id": "default",
            "id": "req_overtime",
            "statement": "Overtime must be traceable",
            "status": "discovery",
            "domainId": "domain_payroll"
        });
        let service = serde_json::json!({
            "schema_version": 1,
            "scope_id": "default",
            "id": "service_payroll_api",
            "name": "payroll-api",
            "description": "Calculates payroll",
            "owner": "platform",
            "repository": "github.com/example/payroll-api",
            "environment": "production",
            "tier": "critical",
            "externalId": "backstage:component/payroll-api",
            "status": "active"
        });
        let service_binding = serde_json::json!({
            "schema_version": 1,
            "scope_id": "default",
            "id": "binding_overtime_payroll_enforces",
            "ruleId": "rule_overtime",
            "serviceId": "service_payroll_api",
            "bindingType": "enforces"
        });

        let domain: Domain = serde_json::from_value(domain).unwrap();
        let requirement: Requirement = serde_json::from_value(requirement).unwrap();
        let service: Service = serde_json::from_value(service).unwrap();
        let service_binding: ServiceBinding = serde_json::from_value(service_binding).unwrap();

        let domain = serde_json::to_value(domain).unwrap();
        let requirement = serde_json::to_value(requirement).unwrap();
        let service = serde_json::to_value(service).unwrap();
        let service_binding = serde_json::to_value(service_binding).unwrap();

        assert_eq!(domain["schema_version"], 1);
        assert_eq!(domain["id"], "domain_payroll");
        assert!(domain.get("createdBy").is_none());
        assert!(domain.get("updatedAt").is_none());
        assert_eq!(requirement["domain_id"], "domain_payroll");
        assert_eq!(service["environment"], "production");
        assert_eq!(service["tier"], "critical");
        assert_eq!(service["external_id"], "backstage:component/payroll-api");
        assert_eq!(service_binding["binding_type"], "enforces");
        assert_eq!(service_binding["rule_id"], "rule_overtime");
    }

    #[test]
    fn question_blocked_on_human_status_accepts_hyphenated_state_and_roundtrips() {
        let question = serde_json::json!({
            "schema_version": 1,
            "scope_id": "default",
            "id": "question_fork",
            "topicId": "topic_overtime",
            "requirementId": "req_overtime",
            "question": "Which UI direction should the shaping map use?",
            "resolutionMethod": "prototype",
            "status": "blocked-on-human",
            "links": []
        });

        let question: Question = serde_json::from_value(question).unwrap();
        let question = serde_json::to_value(question).unwrap();

        assert_eq!(question["status"], "blocked_on_human");
        assert_eq!(question["resolution_method"], "prototype");
    }
}
