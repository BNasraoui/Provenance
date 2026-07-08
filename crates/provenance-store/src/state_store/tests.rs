use super::*;
use provenance_core::{
    ArtifactLink, ArtifactLinkTargetType, EdgeType, IdeationTargetType, IdentityType, Manifest,
    NodeType, QuestionStatus, RepoPathPrefix, SourceReference, TopicStatus, UncertaintyLevel,
};

fn seeded_source_requirement_store() -> (tempfile::TempDir, StateStore, ScopeId) {
    let dir = tempfile::tempdir().unwrap();
    let root = camino::Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
    let layout = ProvenanceLayout::new(root);
    std::fs::create_dir_all(layout.manifest_path().parent().unwrap()).unwrap();
    let scope = ScopeId::new("default").unwrap();
    std::fs::write(
        layout.manifest_path(),
        serde_json::to_string(&Manifest::default_with_scope(
            scope.clone(),
            RepoPathPrefix::new("."),
        ))
        .unwrap(),
    )
    .unwrap();
    let store = StateStore::new(layout);
    store
        .create_source(CreateSourceInput {
            scope_id: scope.clone(),
            id: StableId::new("source_schads").unwrap(),
            name: "SCHADS Award".into(),
            source_type: SourceType::Policy,
            url: None,
            reference: None,
            commit_pin: None,
            effective_date: None,
            review_date: None,
            superseded_by: None,
            origin_thread: None,
            origin_message: None,
        })
        .unwrap();
    store
        .create_requirement(CreateRequirementInput {
            scope_id: scope.clone(),
            id: StableId::new("req_overtime").unwrap(),
            statement: "Overtime".into(),
            description: None,
            status: RequirementStatus::Active,
            domain_id: None,
            origin_thread: None,
            origin_message: None,
        })
        .unwrap();
    (dir, store, scope)
}

#[test]
fn source_requirement_records_are_written_deterministically() {
    let dir = tempfile::tempdir().unwrap();
    let root = camino::Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
    let layout = ProvenanceLayout::new(root);
    std::fs::create_dir_all(layout.manifest_path().parent().unwrap()).unwrap();
    std::fs::write(
        layout.manifest_path(),
        serde_json::to_string(&Manifest::default_with_scope(
            ScopeId::new("default").unwrap(),
            RepoPathPrefix::new("."),
        ))
        .unwrap(),
    )
    .unwrap();
    let store = StateStore::new(layout);
    let scope = ScopeId::new("default").unwrap();
    store
        .create_source(CreateSourceInput {
            scope_id: scope.clone(),
            id: StableId::new("source_schads").unwrap(),
            name: "SCHADS Award".into(),
            source_type: SourceType::Policy,
            url: None,
            reference: None,
            commit_pin: None,
            effective_date: None,
            review_date: None,
            superseded_by: None,
            origin_thread: None,
            origin_message: None,
        })
        .unwrap();
    store
        .create_requirement(CreateRequirementInput {
            scope_id: scope.clone(),
            id: StableId::new("req_overtime").unwrap(),
            statement: "Overtime".into(),
            description: None,
            status: RequirementStatus::Active,
            domain_id: None,
            origin_thread: None,
            origin_message: None,
        })
        .unwrap();
    store
        .add_source_reference(AddSourceReferenceInput {
            scope_id: scope.clone(),
            source_id: StableId::new("source_schads").unwrap(),
            requirement_id: StableId::new("req_overtime").unwrap(),
            clause: None,
        })
        .unwrap();
    assert_eq!(
        store.list_sources(&scope).unwrap()[0].id.as_str(),
        "source_schads"
    );
    assert_eq!(
        store.list_edges().unwrap()[0].edge_type,
        EdgeType::References
    );
}

#[test]
fn concurrent_source_creates_preserve_all_records() {
    let dir = tempfile::tempdir().unwrap();
    let root = camino::Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
    let layout = ProvenanceLayout::new(root);
    std::fs::create_dir_all(layout.manifest_path().parent().unwrap()).unwrap();
    let scope = ScopeId::new("default").unwrap();
    std::fs::write(
        layout.manifest_path(),
        serde_json::to_string(&Manifest::default_with_scope(
            scope.clone(),
            RepoPathPrefix::new("."),
        ))
        .unwrap(),
    )
    .unwrap();
    let store = StateStore::new(layout);

    for index in 0..200 {
        store
            .create_source(CreateSourceInput {
                scope_id: scope.clone(),
                id: StableId::new(format!("source_seed_{index:03}")).unwrap(),
                name: format!("Seed {index:03}"),
                source_type: SourceType::Policy,
                url: None,
                reference: None,
                commit_pin: None,
                effective_date: None,
                review_date: None,
                superseded_by: None,
                origin_thread: None,
                origin_message: None,
            })
            .unwrap();
    }

    let writer_count = 16;
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(writer_count));
    let mut handles = Vec::new();
    for index in 0..writer_count {
        let store = store.clone();
        let scope = scope.clone();
        let barrier = barrier.clone();
        handles.push(std::thread::spawn(move || {
            barrier.wait();
            store
                .create_source(CreateSourceInput {
                    scope_id: scope,
                    id: StableId::new(format!("source_concurrent_{index:03}")).unwrap(),
                    name: format!("Concurrent {index:03}"),
                    source_type: SourceType::Policy,
                    url: None,
                    reference: None,
                    commit_pin: None,
                    effective_date: None,
                    review_date: None,
                    superseded_by: None,
                    origin_thread: None,
                    origin_message: None,
                })
                .unwrap();
        }));
    }
    for handle in handles {
        handle.join().unwrap();
    }

    let sources = store.list_sources(&scope).unwrap();
    assert_eq!(sources.len(), 200 + writer_count);
    for index in 0..writer_count {
        assert!(sources
            .iter()
            .any(|source| { source.id.as_str() == format!("source_concurrent_{index:03}") }));
    }
}

#[test]
fn generic_edges_validate_endpoints_and_delete() {
    let (_dir, store, scope) = seeded_source_requirement_store();
    store
        .create_requirement(CreateRequirementInput {
            scope_id: scope.clone(),
            id: StableId::new("req_leave").unwrap(),
            statement: "Leave".into(),
            description: None,
            status: RequirementStatus::Active,
            domain_id: None,
            origin_thread: None,
            origin_message: None,
        })
        .unwrap();

    let edge = store
        .create_edge(CreateEdgeInput {
            scope_id: scope.clone(),
            edge_type: EdgeType::RefinesInto,
            from_type: NodeType::Requirement,
            from_id: StableId::new("req_overtime").unwrap(),
            to_type: NodeType::Requirement,
            to_id: StableId::new("req_leave").unwrap(),
        })
        .unwrap();

    assert_eq!(edge.edge_type, EdgeType::RefinesInto);
    assert_eq!(store.list_edges().unwrap()[0].id, edge.id);

    let err = store
        .create_edge(CreateEdgeInput {
            scope_id: scope.clone(),
            edge_type: EdgeType::RefinesInto,
            from_type: NodeType::Requirement,
            from_id: StableId::new("req_overtime").unwrap(),
            to_type: NodeType::Requirement,
            to_id: StableId::new("req_missing").unwrap(),
        })
        .unwrap_err();
    assert!(err.to_string().contains("to endpoint does not exist"));

    let deleted = store.delete_edge(&scope, &edge.id).unwrap();
    assert_eq!(deleted.id, edge.id);
    assert!(store.list_edges().unwrap().is_empty());
}

#[test]
fn shaping_records_are_written_deterministically_and_validate_relationships() {
    let (_dir, store, scope) = seeded_source_requirement_store();

    store
        .create_topic(CreateTopicInput {
            scope_id: scope.clone(),
            id: StableId::new("topic_b").unwrap(),
            requirement_id: StableId::new("req_overtime").unwrap(),
            title: "B topic".into(),
            status: TopicStatus::Open,
            links: Vec::new(),
        })
        .unwrap();
    store
        .create_topic(CreateTopicInput {
            scope_id: scope.clone(),
            id: StableId::new("topic_a").unwrap(),
            requirement_id: StableId::new("req_overtime").unwrap(),
            title: "A topic".into(),
            status: TopicStatus::Explored,
            links: vec![
                ArtifactLink {
                    target_type: ArtifactLinkTargetType::Source,
                    target_id: StableId::new("source_schads").unwrap(),
                },
                ArtifactLink {
                    target_type: ArtifactLinkTargetType::Requirement,
                    target_id: StableId::new("req_overtime").unwrap(),
                },
                ArtifactLink {
                    target_type: ArtifactLinkTargetType::Source,
                    target_id: StableId::new("source_schads").unwrap(),
                },
            ],
        })
        .unwrap();
    store
        .create_boundary(CreateBoundaryInput {
            scope_id: scope.clone(),
            id: StableId::new("boundary_no_manual_rework").unwrap(),
            requirement_id: StableId::new("req_overtime").unwrap(),
            statement: "No manual rework".into(),
            source_ref: Some(SourceReference {
                source_id: StableId::new("source_schads").unwrap(),
                clause: Some("28.1".into()),
            }),
        })
        .unwrap();
    let question = store
        .create_question(CreateQuestionInput {
            scope_id: scope.clone(),
            id: StableId::new("question_threshold").unwrap(),
            topic_id: StableId::new("topic_a").unwrap(),
            question: "Which threshold applies?".into(),
            resolution_method: ResolutionMethod::Grill,
            status: QuestionStatus::Open,
            answer: None,
            links: Vec::new(),
            resolution_id: None,
        })
        .unwrap();

    let topics = store.list_topics(&scope).unwrap();
    assert_eq!(topics[0].id.as_str(), "topic_a");
    assert_eq!(topics[0].links.len(), 2);
    assert_eq!(topics[0].links[0].target_id.as_str(), "req_overtime");
    assert_eq!(topics[0].links[1].target_id.as_str(), "source_schads");
    assert_eq!(
        store.list_boundaries(&scope).unwrap()[0]
            .source_ref
            .as_ref()
            .unwrap()
            .clause
            .as_deref(),
        Some("28.1")
    );
    assert_eq!(question.requirement_id.as_str(), "req_overtime");
    assert!(store
        .create_question(CreateQuestionInput {
            scope_id: scope,
            id: StableId::new("question_missing_topic").unwrap(),
            topic_id: StableId::new("topic_missing").unwrap(),
            question: "Missing topic?".into(),
            resolution_method: ResolutionMethod::Grill,
            status: QuestionStatus::Open,
            answer: None,
            links: Vec::new(),
            resolution_id: None,
        })
        .unwrap_err()
        .to_string()
        .contains("topic does not exist"));
}

#[test]
fn topic_claims_are_check_and_set_and_clear_on_close() {
    let (_dir, store, scope) = seeded_source_requirement_store();
    store
        .create_topic(CreateTopicInput {
            scope_id: scope.clone(),
            id: StableId::new("topic_overtime").unwrap(),
            requirement_id: StableId::new("req_overtime").unwrap(),
            title: "Overtime eligibility".into(),
            status: TopicStatus::Open,
            links: Vec::new(),
        })
        .unwrap();
    let topic_id = StableId::new("topic_overtime").unwrap();

    let claimed = store.claim_topic(&scope, &topic_id, "agent-one").unwrap();
    assert_eq!(claimed.claimed_by.as_deref(), Some("agent-one"));
    assert!(claimed.claimed_at.unwrap() > 0);

    let err = store
        .claim_topic(&scope, &topic_id, "agent-two")
        .unwrap_err();
    assert!(err
        .to_string()
        .contains("topic topic_overtime is already claimed by agent-one"));

    let released = store.release_topic(&scope, &topic_id).unwrap();
    assert_eq!(released.claimed_by, None);
    assert_eq!(released.claimed_at, None);
    assert!(store
        .release_topic(&scope, &topic_id)
        .unwrap_err()
        .to_string()
        .contains("topic topic_overtime is not claimed"));

    store.claim_topic(&scope, &topic_id, "agent-two").unwrap();
    let closed = store.close_topic(&scope, &topic_id).unwrap();
    assert_eq!(closed.status, TopicStatus::Closed);
    assert_eq!(closed.claimed_by, None);
    assert_eq!(closed.claimed_at, None);
    assert!(store
        .claim_topic(&scope, &topic_id, "agent-one")
        .unwrap_err()
        .to_string()
        .contains("closed"));
    assert_eq!(
        store.list_topics(&scope).unwrap()[0].status,
        TopicStatus::Closed
    );
}

#[test]
fn question_claims_clear_when_answered() {
    let (_dir, store, scope) = seeded_source_requirement_store();
    store
        .create_topic(CreateTopicInput {
            scope_id: scope.clone(),
            id: StableId::new("topic_overtime").unwrap(),
            requirement_id: StableId::new("req_overtime").unwrap(),
            title: "Overtime eligibility".into(),
            status: TopicStatus::Open,
            links: Vec::new(),
        })
        .unwrap();
    store
        .create_question(CreateQuestionInput {
            scope_id: scope.clone(),
            id: StableId::new("question_threshold").unwrap(),
            topic_id: StableId::new("topic_overtime").unwrap(),
            question: "Which threshold applies?".into(),
            resolution_method: ResolutionMethod::Research,
            status: QuestionStatus::Open,
            answer: None,
            links: Vec::new(),
            resolution_id: None,
        })
        .unwrap();
    let question_id = StableId::new("question_threshold").unwrap();

    let claimed = store
        .claim_question(&scope, &question_id, "agent-one")
        .unwrap();
    assert_eq!(claimed.claimed_by.as_deref(), Some("agent-one"));
    assert_eq!(claimed.resolution_method, ResolutionMethod::Research);
    assert!(store
        .claim_question(&scope, &question_id, "agent-two")
        .unwrap_err()
        .to_string()
        .contains("question question_threshold is already claimed by agent-one"));

    let answered = store
        .answer_question(
            &scope,
            &question_id,
            "Use the SCHADS threshold.".into(),
            None,
        )
        .unwrap();
    assert_eq!(answered.status, QuestionStatus::Answered);
    assert_eq!(
        answered.answer.as_deref(),
        Some("Use the SCHADS threshold.")
    );
    assert_eq!(answered.claimed_by, None);
    assert_eq!(answered.claimed_at, None);
    assert!(store
        .claim_question(&scope, &question_id, "agent-two")
        .unwrap_err()
        .to_string()
        .contains("answered"));

    let persisted = &store.list_questions(&scope).unwrap()[0];
    assert_eq!(persisted.status, QuestionStatus::Answered);
    assert_eq!(persisted.claimed_by, None);
}

#[test]
fn requirement_fog_is_set_and_cleared_as_free_text() {
    let (_dir, store, scope) = seeded_source_requirement_store();
    let requirement_id = StableId::new("req_overtime").unwrap();

    let updated = store
        .set_requirement_fog(
            &scope,
            &requirement_id,
            Some("something about public holidays and sleepovers".into()),
        )
        .unwrap();
    assert_eq!(
        updated.fog.as_deref(),
        Some("something about public holidays and sleepovers")
    );
    assert_eq!(
        store.list_requirements(&scope).unwrap()[0].fog.as_deref(),
        Some("something about public holidays and sleepovers")
    );

    let cleared = store
        .set_requirement_fog(&scope, &requirement_id, None)
        .unwrap();
    assert_eq!(cleared.fog, None);
    assert!(store
        .set_requirement_fog(&scope, &StableId::new("req_missing").unwrap(), None)
        .unwrap_err()
        .to_string()
        .contains("requirement does not exist"));
}

#[test]
fn ideation_output_records_are_written_deterministically() {
    let dir = tempfile::tempdir().unwrap();
    let root = camino::Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
    let layout = ProvenanceLayout::new(root);
    std::fs::create_dir_all(layout.manifest_path().parent().unwrap()).unwrap();
    std::fs::write(
        layout.manifest_path(),
        serde_json::to_string(&Manifest::default_with_scope(
            ScopeId::new("default").unwrap(),
            RepoPathPrefix::new("."),
        ))
        .unwrap(),
    )
    .unwrap();
    let store = StateStore::new(layout);
    let scope = ScopeId::new("default").unwrap();

    store
        .create_contribution(CreateContributionInput {
            scope_id: scope.clone(),
            id: StableId::new("contrib_b").unwrap(),
            target: IdeationTarget {
                artifact_type: IdeationTargetType::Requirement,
                artifact_id: StableId::new("req_overtime").unwrap(),
            },
            participant_slot: "reviewer".into(),
            stance: ContributionStance::Support,
            strongest_finding: "Supported by evidence".into(),
            evidence_references: Vec::new(),
            material_claims: Vec::new(),
            risks: Vec::new(),
            objections: Vec::new(),
            challenges: Vec::new(),
            suggested_artifact_changes: Vec::new(),
            unsupported_recommendations: Vec::new(),
            uncertainty: UncertaintyRating {
                level: UncertaintyLevel::Low,
                rationale: "Direct evidence".into(),
            },
            open_questions: Vec::new(),
        })
        .unwrap();
    store
        .create_contribution(CreateContributionInput {
            scope_id: scope.clone(),
            id: StableId::new("contrib_a").unwrap(),
            target: IdeationTarget {
                artifact_type: IdeationTargetType::Requirement,
                artifact_id: StableId::new("req_overtime").unwrap(),
            },
            participant_slot: "refuter".into(),
            stance: ContributionStance::NeedsMoreEvidence,
            strongest_finding: "Needs more evidence".into(),
            evidence_references: Vec::new(),
            material_claims: Vec::new(),
            risks: Vec::new(),
            objections: Vec::new(),
            challenges: Vec::new(),
            suggested_artifact_changes: Vec::new(),
            unsupported_recommendations: Vec::new(),
            uncertainty: UncertaintyRating {
                level: UncertaintyLevel::High,
                rationale: "Missing source".into(),
            },
            open_questions: Vec::new(),
        })
        .unwrap();

    assert_eq!(
        store.list_contributions(&scope).unwrap()[0].id.as_str(),
        "contrib_a"
    );
}

#[test]
fn proposal_state_requires_duplicate_or_superseded_link() {
    let dir = tempfile::tempdir().unwrap();
    let root = camino::Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
    let layout = ProvenanceLayout::new(root);
    std::fs::create_dir_all(layout.manifest_path().parent().unwrap()).unwrap();
    std::fs::write(
        layout.manifest_path(),
        serde_json::to_string(&Manifest::default_with_scope(
            ScopeId::new("default").unwrap(),
            RepoPathPrefix::new("."),
        ))
        .unwrap(),
    )
    .unwrap();
    let store = StateStore::new(layout);
    let scope = ScopeId::new("default").unwrap();

    let err = store
        .create_proposal_card(CreateProposalCardInput {
            scope_id: scope,
            id: StableId::new("proposal_duplicate").unwrap(),
            proposal_key: "duplicate".into(),
            proposal_type: ProposalType::RequirementCandidate,
            title: "Duplicate proposal".into(),
            summary: "This should point at the original proposal.".into(),
            confidence: None,
            traceability: ProposalTraceability {
                target: IdeationTarget {
                    artifact_type: IdeationTargetType::Requirement,
                    artifact_id: StableId::new("req_overtime").unwrap(),
                },
                source_ids: Vec::new(),
                evidence_references: Vec::new(),
                supporting_claim_ids: Vec::new(),
            },
            promotion_state: PromotionState::Duplicate,
            duplicate_of: None,
            superseded_by: None,
        })
        .unwrap_err();

    assert!(err
        .to_string()
        .contains("duplicate proposals must set duplicate_of"));
}

#[test]
fn promotion_decision_updates_proposal_state() {
    let dir = tempfile::tempdir().unwrap();
    let root = camino::Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
    let layout = ProvenanceLayout::new(root);
    std::fs::create_dir_all(layout.manifest_path().parent().unwrap()).unwrap();
    std::fs::write(
        layout.manifest_path(),
        serde_json::to_string(&Manifest::default_with_scope(
            ScopeId::new("default").unwrap(),
            RepoPathPrefix::new("."),
        ))
        .unwrap(),
    )
    .unwrap();
    let store = StateStore::new(layout);
    let scope = ScopeId::new("default").unwrap();

    store
        .create_proposal_card(CreateProposalCardInput {
            scope_id: scope.clone(),
            id: StableId::new("proposal_overtime").unwrap(),
            proposal_key: "overtime".into(),
            proposal_type: ProposalType::RequirementCandidate,
            title: "Clarify overtime".into(),
            summary: "Clarify the overtime requirement.".into(),
            confidence: None,
            traceability: ProposalTraceability {
                target: IdeationTarget {
                    artifact_type: IdeationTargetType::Requirement,
                    artifact_id: StableId::new("req_overtime").unwrap(),
                },
                source_ids: Vec::new(),
                evidence_references: Vec::new(),
                supporting_claim_ids: Vec::new(),
            },
            promotion_state: PromotionState::Proposed,
            duplicate_of: None,
            superseded_by: None,
        })
        .unwrap();
    store
        .create_promotion_decision(CreatePromotionDecisionInput {
            scope_id: scope.clone(),
            id: StableId::new("decision_overtime").unwrap(),
            proposal_id: StableId::new("proposal_overtime").unwrap(),
            decision: PromotionDecision::Accepted,
            rationale: "Approved by human review.".into(),
            actor: PromotionActor {
                identity_type: IdentityType::Human,
                id: "ben".into(),
                name: None,
            },
            canonical_artifact: None,
        })
        .unwrap();

    assert_eq!(
        store.list_proposal_cards(&scope).unwrap()[0].promotion_state,
        PromotionState::Accepted
    );
}

fn proposal_input(
    scope: &ScopeId,
    id: &str,
    title: &str,
    promotion_state: PromotionState,
) -> CreateProposalCardInput {
    CreateProposalCardInput {
        scope_id: scope.clone(),
        id: StableId::new(id).unwrap(),
        proposal_key: "overtime".into(),
        proposal_type: ProposalType::RequirementCandidate,
        title: title.into(),
        summary: "Clarify the overtime requirement.".into(),
        confidence: None,
        traceability: ProposalTraceability {
            target: IdeationTarget {
                artifact_type: IdeationTargetType::Requirement,
                artifact_id: StableId::new("req_overtime").unwrap(),
            },
            source_ids: Vec::new(),
            evidence_references: Vec::new(),
            supporting_claim_ids: Vec::new(),
        },
        promotion_state,
        duplicate_of: None,
        superseded_by: None,
    }
}

#[test]
fn replacing_accepted_proposal_reports_human_disposition() {
    let (_dir, store, scope) = seeded_source_requirement_store();
    store
        .create_proposal_card(proposal_input(
            &scope,
            "proposal_overtime",
            "Original proposal",
            PromotionState::Accepted,
        ))
        .unwrap();

    let err = store
        .upsert_proposal_card(proposal_input(
            &scope,
            "proposal_overtime",
            "Replacement proposal",
            PromotionState::Proposed,
        ))
        .unwrap_err();
    let message = err.to_string();

    assert!(message.contains("human disposition"));
    assert!(message.contains("accepted"));
}

#[test]
fn replacing_proposed_proposal_with_decision_edge_reports_human_disposition() {
    let (_dir, store, scope) = seeded_source_requirement_store();
    store
        .create_proposal_card(proposal_input(
            &scope,
            "proposal_overtime",
            "Original proposal",
            PromotionState::Proposed,
        ))
        .unwrap();
    store
        .create_promotion_decision(CreatePromotionDecisionInput {
            scope_id: scope.clone(),
            id: StableId::new("decision_overtime").unwrap(),
            proposal_id: StableId::new("proposal_overtime").unwrap(),
            decision: PromotionDecision::Accepted,
            rationale: "Approved by human review.".into(),
            actor: PromotionActor {
                identity_type: IdentityType::Human,
                id: "ben".into(),
                name: None,
            },
            canonical_artifact: None,
        })
        .unwrap();
    let path = crate::shards::proposal_cards_path(&store.layout, &scope);
    store
        .mutate_jsonl_records(&path, |proposals: &mut Vec<ProposalCard>| {
            proposals[0].promotion_state = PromotionState::Proposed;
            Ok(())
        })
        .unwrap();

    let err = store
        .upsert_proposal_card(proposal_input(
            &scope,
            "proposal_overtime",
            "Replacement proposal",
            PromotionState::Proposed,
        ))
        .unwrap_err();
    let message = err.to_string();

    assert!(message.contains("human disposition"));
    assert!(message.contains("proposed"));
    assert!(message.contains("decision_overtime"));
}
