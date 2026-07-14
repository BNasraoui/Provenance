use super::super::*;
use super::fixtures::*;
use crate::state_store::{
    CreateQuestionInput, CreateResolutionInput, CreateSourceInput, CreateTopicInput, StateStore,
};
use provenance_core::{
    QuestionStatus, ResolutionInput, ResolutionInputType, ResolutionMethod, ResolutionStatus,
    SourceType, TopicStatus,
};

#[tokio::test]
async fn materialize_state_caches_fog_resolution_method_and_claim_state() {
    let (_dir, layout, scope) = seeded_layout();
    let store = StateStore::new(layout.clone());
    store
        .set_requirement_fog(
            &scope,
            &sid("req_schads_overtime"),
            Some("sleepover rules; something about broken shifts".into()),
        )
        .unwrap();
    store
        .create_topic(CreateTopicInput {
            scope_id: scope.clone(),
            id: sid("topic_overtime"),
            requirement_id: sid("req_schads_overtime"),
            title: "Overtime eligibility".into(),
            status: TopicStatus::Open,
            links: Vec::new(),
        })
        .unwrap();
    store
        .create_question(CreateQuestionInput {
            scope_id: scope.clone(),
            id: sid("question_threshold"),
            topic_id: sid("topic_overtime"),
            question: "Which threshold applies?".into(),
            resolution_method: ResolutionMethod::Verify,
            status: QuestionStatus::Open,
            answer: None,
            links: Vec::new(),
            resolution_id: None,
        })
        .unwrap();
    store
        .claim_topic(&scope, &sid("topic_overtime"), "agent-one")
        .unwrap();
    store
        .claim_question(&scope, &sid("question_threshold"), "agent-two")
        .unwrap();

    materialize_state(&layout).await.unwrap();
    let pool = open_cache(&layout).await.unwrap();
    let fog: Option<String> = sqlx::query_scalar("SELECT fog FROM requirements WHERE id = ?")
        .bind("req_schads_overtime")
        .fetch_one(&pool)
        .await
        .unwrap();
    let topic: (Option<String>, Option<i64>) =
        sqlx::query_as("SELECT claimed_by, claimed_at FROM topics WHERE id = ?")
            .bind("topic_overtime")
            .fetch_one(&pool)
            .await
            .unwrap();
    let question: (String, Option<String>, Option<i64>) = sqlx::query_as(
        "SELECT resolution_method, claimed_by, claimed_at FROM questions WHERE id = ?",
    )
    .bind("question_threshold")
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        fog.as_deref(),
        Some("sleepover rules; something about broken shifts")
    );
    assert_eq!(topic.0.as_deref(), Some("agent-one"));
    assert!(topic.1.unwrap() > 0);
    assert_eq!(question.0, "verify");
    assert_eq!(question.1.as_deref(), Some("agent-two"));
    assert!(question.2.unwrap() > 0);
}

#[tokio::test]
async fn materialize_state_caches_enriched_source_and_resolution_fields() {
    let (_dir, layout, scope) = empty_layout();
    let store = StateStore::new(layout.clone());
    store
        .create_source(CreateSourceInput {
            scope_id: scope.clone(),
            id: sid("source_sah"),
            name: "Support at Home".into(),
            source_type: SourceType::Legislation,
            url: Some("https://example.test/sah".into()),
            reference: Some("Department guidance".into()),
            commit_pin: None,
            effective_date: Some(1_714_521_600_000),
            review_date: Some(1_717_200_000_000),
            superseded_by: Some(sid("source_sah_2025")),
            origin_thread: None,
            origin_message: None,
        })
        .unwrap();
    store
        .create_resolution(CreateResolutionInput {
            scope_id: scope,
            id: sid("res_sah"),
            title: "SAH extraction".into(),
            requirement_id: None,
            position: "Keep as draft extraction".into(),
            rationale: "Needs human review".into(),
            status: ResolutionStatus::Draft,
            context: Some("Codebase scan".into()),
            enforcement: Some("specification".into()),
            confidence: Some(0.91),
            inputs: vec![ResolutionInput {
                input_type: ResolutionInputType::Regulatory,
                reference: "SAH program manual".into(),
                summary: "Program rules reviewed".into(),
            }],
            made_by: Some("Analyst One".into()),
            approved_by: Some("Approver Two".into()),
            approved_at: Some(1_714_780_800_000),
            superseded_by: Some(sid("res_sah_2025")),
            origin_thread: None,
            origin_message: None,
        })
        .unwrap();
    materialize_state(&layout).await.unwrap();
    let pool = open_cache(&layout).await.unwrap();
    let source: (Option<String>, Option<i64>, Option<i64>, Option<String>) = sqlx::query_as(
        "SELECT reference, effective_date, review_date, superseded_by FROM sources WHERE id = ?",
    )
    .bind("source_sah")
    .fetch_one(&pool)
    .await
    .unwrap();
    let resolution: (String, Option<String>, Option<String>, Option<i64>, Option<String>) = sqlx::query_as("SELECT inputs, made_by, approved_by, approved_at, superseded_by FROM resolutions WHERE id = ?").bind("res_sah").fetch_one(&pool).await.unwrap();
    assert_eq!(source.0.as_deref(), Some("Department guidance"));
    assert_eq!(source.1, Some(1_714_521_600_000));
    assert_eq!(source.2, Some(1_717_200_000_000));
    assert_eq!(source.3.as_deref(), Some("source_sah_2025"));
    assert!(resolution.0.contains(r#""input_type":"regulatory""#));
    assert_eq!(resolution.1.as_deref(), Some("Analyst One"));
    assert_eq!(resolution.2.as_deref(), Some("Approver Two"));
    assert_eq!(resolution.3, Some(1_714_780_800_000));
    assert_eq!(resolution.4.as_deref(), Some("res_sah_2025"));
}

#[tokio::test]
async fn materialize_state_caches_commit_pin_and_confidence_scores() {
    let (_dir, layout, scope) = empty_layout();
    let sources_path = crate::shards::sources_path(&layout, &scope);
    std::fs::create_dir_all(sources_path.parent().unwrap()).unwrap();
    std::fs::write(&sources_path, r#"{"schema_version":1,"scope_id":"default","id":"source_codebase","name":"Example API","source_type":"project_artifact","commit_pin":"5e1f2a9c4b6d8e0f1234567890abcdef12345678"}
"#).unwrap();
    let contributions_path = crate::shards::contributions_path(&layout, &scope);
    std::fs::create_dir_all(contributions_path.parent().unwrap()).unwrap();
    std::fs::write(&contributions_path, r#"{"schema_version":1,"scope_id":"default","id":"contrib_reviewer_001","target":{"artifact_type":"requirement","artifact_id":"req_overtime"},"participant_slot":"reviewer","stance":"support","strongest_finding":"Supported by code evidence.","evidence_references":[],"material_claims":[{"claim_id":"claim_overtime_threshold","statement":"Overtime starts after the award threshold.","evidence_type":"artifact","evidence_reference_ids":[],"confidence":0.87}],"risks":[],"objections":[],"challenges":[],"suggested_artifact_changes":[],"unsupported_recommendations":[],"uncertainty":{"level":"low","rationale":"Direct code evidence."},"open_questions":[]}
"#).unwrap();
    let proposals_path = crate::shards::proposal_cards_path(&layout, &scope);
    std::fs::create_dir_all(proposals_path.parent().unwrap()).unwrap();
    std::fs::write(&proposals_path, r#"{"schema_version":1,"scope_id":"default","id":"proposal_overtime_traceability","proposal_key":"req-overtime-traceability","proposal_type":"requirement_candidate","title":"Clarify overtime traceability","summary":"Add source-backed threshold language.","confidence":0.83,"traceability":{"target":{"artifact_type":"requirement","artifact_id":"req_overtime"},"source_ids":["source_codebase"],"evidence_references":[],"supporting_claim_ids":["claim_overtime_threshold"]},"promotion_state":"proposed"}
"#).unwrap();
    materialize_state(&layout).await.unwrap();
    let pool = open_cache(&layout).await.unwrap();
    let commit_pin: Option<String> =
        sqlx::query_scalar("SELECT commit_pin FROM sources WHERE id = ?")
            .bind("source_codebase")
            .fetch_one(&pool)
            .await
            .unwrap();
    let confidence: Option<f64> =
        sqlx::query_scalar("SELECT confidence FROM proposal_cards WHERE id = ?")
            .bind("proposal_overtime_traceability")
            .fetch_one(&pool)
            .await
            .unwrap();
    let builds_on: String = sqlx::query_scalar("SELECT builds_on FROM proposal_cards WHERE id = ?")
        .bind("proposal_overtime_traceability")
        .fetch_one(&pool)
        .await
        .unwrap();
    let payload: String = sqlx::query_scalar("SELECT payload FROM contributions WHERE id = ?")
        .bind("contrib_reviewer_001")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        commit_pin.as_deref(),
        Some("5e1f2a9c4b6d8e0f1234567890abcdef12345678")
    );
    assert_eq!(confidence, Some(0.83));
    assert_eq!(builds_on, "[]");
    assert!(payload.contains(r#""confidence":0.87"#));
}
