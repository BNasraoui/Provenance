mod health;
mod impact;
mod prime;
mod traceability;

pub use health::*;
pub use impact::*;
pub use prime::*;
pub use traceability::*;

use crate::{layout::ProvenanceLayout, migrations, state_store::StateStore};
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct MaterializeReport {
    pub records_loaded: u64,
    pub migrations_applied: Vec<String>,
}

pub async fn open_cache(layout: &ProvenanceLayout) -> anyhow::Result<SqlitePool> {
    std::fs::create_dir_all(layout.cache_dir())?;
    let options = SqliteConnectOptions::from_str(&format!("sqlite://{}", layout.cache_db_path()))?
        .create_if_missing(true);
    Ok(SqlitePool::connect_with(options).await?)
}

pub async fn materialize_empty_state(
    layout: &ProvenanceLayout,
) -> anyhow::Result<MaterializeReport> {
    let pool = open_cache(layout).await?;
    let migrations_applied = migrations::run_migrations(&pool).await?;
    Ok(MaterializeReport {
        records_loaded: 0,
        migrations_applied,
    })
}

#[allow(clippy::too_many_lines)]
pub async fn materialize_state(layout: &ProvenanceLayout) -> anyhow::Result<MaterializeReport> {
    let pool = open_cache(layout).await?;
    let migrations_applied = migrations::run_migrations(&pool).await?;
    let store = StateStore::new(layout.clone());
    let manifest = store.manifest()?;
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM sources").execute(&mut *tx).await?;
    sqlx::query("DELETE FROM domains").execute(&mut *tx).await?;
    sqlx::query("DELETE FROM requirements")
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM boundaries")
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM topics").execute(&mut *tx).await?;
    sqlx::query("DELETE FROM questions")
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM edges").execute(&mut *tx).await?;
    sqlx::query("DELETE FROM resolutions")
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM rules").execute(&mut *tx).await?;
    sqlx::query("DELETE FROM services")
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM service_bindings")
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM messages")
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM threads").execute(&mut *tx).await?;
    sqlx::query("DELETE FROM contributions")
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM synthesis_packets")
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM proposal_cards")
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM promotion_decisions")
        .execute(&mut *tx)
        .await?;
    let mut records_loaded = 0;
    for scope in &manifest.scopes {
        for source in store.list_sources(&scope.id)? {
            sqlx::query(
                "INSERT INTO sources (scope_id, id, name, source_type, url, reference, commit_pin, effective_date, review_date, superseded_by) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(source.scope_id.as_str())
            .bind(source.id.as_str())
            .bind(source.name)
            .bind(serde_name(&source.source_type)?)
            .bind(source.url)
            .bind(source.reference)
            .bind(source.commit_pin)
            .bind(source.effective_date)
            .bind(source.review_date)
            .bind(
                source
                    .superseded_by
                    .as_ref()
                    .map(provenance_core::StableId::as_str),
            )
            .execute(&mut *tx)
            .await?;
            records_loaded += 1;
        }
        for requirement in store.list_requirements(&scope.id)? {
            sqlx::query(
                "INSERT INTO requirements (scope_id, id, statement, status, domain_id, fog) VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(requirement.scope_id.as_str())
            .bind(requirement.id.as_str())
            .bind(requirement.statement)
            .bind(serde_name(&requirement.status)?)
            .bind(
                requirement
                    .domain_id
                    .as_ref()
                    .map(provenance_core::StableId::as_str),
            )
            .bind(requirement.fog)
            .execute(&mut *tx)
            .await?;
            records_loaded += 1;
        }
        for domain in store.list_domains(&scope.id)? {
            sqlx::query(
                "INSERT INTO domains (scope_id, id, name, description, color) VALUES (?, ?, ?, ?, ?)",
            )
            .bind(domain.scope_id.as_str())
            .bind(domain.id.as_str())
            .bind(domain.name)
            .bind(domain.description)
            .bind(domain.color)
            .execute(&mut *tx)
            .await?;
            records_loaded += 1;
        }
        for boundary in store.list_boundaries(&scope.id)? {
            let source_id = boundary
                .source_ref
                .as_ref()
                .map(|source_ref| source_ref.source_id.as_str());
            let source_clause = boundary
                .source_ref
                .as_ref()
                .and_then(|source_ref| source_ref.clause.as_deref());
            sqlx::query("INSERT INTO boundaries (scope_id, id, requirement_id, statement, source_id, source_clause) VALUES (?, ?, ?, ?, ?, ?)")
                .bind(boundary.scope_id.as_str())
                .bind(boundary.id.as_str())
                .bind(boundary.requirement_id.as_str())
                .bind(boundary.statement)
                .bind(source_id)
                .bind(source_clause)
                .execute(&mut *tx)
                .await?;
            records_loaded += 1;
        }
        for topic in store.list_topics(&scope.id)? {
            sqlx::query("INSERT INTO topics (scope_id, id, requirement_id, title, status, claimed_by, claimed_at, links) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
                .bind(topic.scope_id.as_str())
                .bind(topic.id.as_str())
                .bind(topic.requirement_id.as_str())
                .bind(topic.title)
                .bind(serde_name(&topic.status)?)
                .bind(topic.claimed_by)
                .bind(topic.claimed_at)
                .bind(serde_json::to_string(&topic.links)?)
                .execute(&mut *tx)
                .await?;
            records_loaded += 1;
        }
        for question in store.list_questions(&scope.id)? {
            sqlx::query("INSERT INTO questions (scope_id, id, topic_id, requirement_id, question, resolution_method, status, claimed_by, claimed_at, answer, links, resolution_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
                .bind(question.scope_id.as_str())
                .bind(question.id.as_str())
                .bind(question.topic_id.as_str())
                .bind(question.requirement_id.as_str())
                .bind(question.question)
                .bind(serde_name(&question.resolution_method)?)
                .bind(serde_name(&question.status)?)
                .bind(question.claimed_by)
                .bind(question.claimed_at)
                .bind(question.answer)
                .bind(serde_json::to_string(&question.links)?)
                .bind(question.resolution_id.as_ref().map(provenance_core::StableId::as_str))
                .execute(&mut *tx)
                .await?;
            records_loaded += 1;
        }
        for resolution in store.list_resolutions(&scope.id)? {
            sqlx::query("INSERT INTO resolutions (scope_id, id, title, position, rationale, status, review_on, review_triggers, context, enforcement, confidence, inputs, made_by, approved_by, approved_at, superseded_by) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
                .bind(resolution.scope_id.as_str())
                .bind(resolution.id.as_str())
                .bind(resolution.title)
                .bind(resolution.position)
                .bind(resolution.rationale)
                .bind(serde_name(&resolution.status)?)
                .bind(resolution.review_on)
                .bind(resolution.review_triggers.to_string())
                .bind(resolution.context)
                .bind(resolution.enforcement)
                .bind(resolution.confidence)
                .bind(serde_json::to_string(&resolution.inputs)?)
                .bind(resolution.made_by)
                .bind(resolution.approved_by)
                .bind(resolution.approved_at)
                .bind(
                    resolution
                        .superseded_by
                        .as_ref()
                        .map(provenance_core::StableId::as_str),
                )
                .execute(&mut *tx)
                .await?;
            records_loaded += 1;
        }
        for rule in store.list_rules(&scope.id)? {
            sqlx::query("INSERT INTO rules (scope_id, id, rule_code, statement, status, severity, expression, inputs) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
                .bind(rule.scope_id.as_str())
                .bind(rule.id.as_str())
                .bind(rule.rule_code)
                .bind(rule.statement)
                .bind(serde_name(&rule.status)?)
                .bind(serde_name(&rule.severity)?)
                .bind(rule.expression.to_string())
                .bind(rule.inputs.to_string())
                .execute(&mut *tx)
                .await?;
            records_loaded += 1;
        }
        for service in store.list_services(&scope.id)? {
            sqlx::query("INSERT INTO services (scope_id, id, name, description, owner, repository, environment, tier, external_id, status) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
                .bind(service.scope_id.as_str())
                .bind(service.id.as_str())
                .bind(service.name)
                .bind(service.description)
                .bind(service.owner)
                .bind(service.repository)
                .bind(service.environment.as_ref().map(serde_name).transpose()?)
                .bind(service.tier.as_ref().map(serde_name).transpose()?)
                .bind(service.external_id)
                .bind(serde_name(&service.status)?)
                .execute(&mut *tx)
                .await?;
            records_loaded += 1;
        }
        for binding in store.list_service_bindings(&scope.id)? {
            sqlx::query("INSERT INTO service_bindings (scope_id, id, rule_id, service_id, binding_type) VALUES (?, ?, ?, ?, ?)")
                .bind(binding.scope_id.as_str())
                .bind(binding.id.as_str())
                .bind(binding.rule_id.as_str())
                .bind(binding.service_id.as_str())
                .bind(serde_name(&binding.binding_type)?)
                .execute(&mut *tx)
                .await?;
            records_loaded += 1;
        }
        for thread in store.list_threads(&scope.id)? {
            sqlx::query("INSERT INTO threads (scope_id, id, parent_type, parent_id, status, created_at) VALUES (?, ?, ?, ?, ?, ?)")
                .bind(thread.scope_id.as_str())
                .bind(thread.id.as_str())
                .bind(serde_name(&thread.parent.node_type)?)
                .bind(thread.parent.node_id.as_str())
                .bind(serde_name(&thread.status)?)
                .bind(thread.created_at)
                .execute(&mut *tx)
                .await?;
            records_loaded += 1;
        }
        for message in store.list_messages(&scope.id)? {
            sqlx::query("INSERT INTO messages (scope_id, id, thread_id, role, body, created_at, ai_metadata) VALUES (?, ?, ?, ?, ?, ?, ?)")
                .bind(message.scope_id.as_str())
                .bind(message.id.as_str())
                .bind(message.thread_id.as_str())
                .bind(serde_name(&message.role)?)
                .bind(message.body)
                .bind(message.created_at)
                .bind(message.ai_metadata.map(|value| value.to_string()))
                .execute(&mut *tx)
                .await?;
            records_loaded += 1;
        }
        for contribution in store.list_contributions(&scope.id)? {
            sqlx::query("INSERT INTO contributions (scope_id, id, target_type, target_id, participant_slot, stance, strongest_finding, uncertainty, payload) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
                .bind(contribution.scope_id.as_str())
                .bind(contribution.id.as_str())
                .bind(serde_name(&contribution.target.artifact_type)?)
                .bind(contribution.target.artifact_id.as_str())
                .bind(&contribution.participant_slot)
                .bind(serde_name(&contribution.stance)?)
                .bind(&contribution.strongest_finding)
                .bind(serde_json::to_string(&contribution.uncertainty)?)
                .bind(serde_json::to_string(&contribution)?)
                .execute(&mut *tx)
                .await?;
            records_loaded += 1;
        }
        for synthesis_packet in store.list_synthesis_packets(&scope.id)? {
            sqlx::query("INSERT INTO synthesis_packets (scope_id, id, target_type, target_id, summary, payload) VALUES (?, ?, ?, ?, ?, ?)")
                .bind(synthesis_packet.scope_id.as_str())
                .bind(synthesis_packet.id.as_str())
                .bind(serde_name(&synthesis_packet.target.artifact_type)?)
                .bind(synthesis_packet.target.artifact_id.as_str())
                .bind(&synthesis_packet.summary)
                .bind(serde_json::to_string(&synthesis_packet)?)
                .execute(&mut *tx)
                .await?;
            records_loaded += 1;
        }
        for proposal in store.list_proposal_cards(&scope.id)? {
            sqlx::query("INSERT INTO proposal_cards (scope_id, id, proposal_key, proposal_type, title, summary, confidence, target_type, target_id, traceability, promotion_state, duplicate_of, superseded_by) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
                .bind(proposal.scope_id.as_str())
                .bind(proposal.id.as_str())
                .bind(&proposal.proposal_key)
                .bind(serde_name(&proposal.proposal_type)?)
                .bind(&proposal.title)
                .bind(&proposal.summary)
                .bind(proposal.confidence)
                .bind(serde_name(&proposal.traceability.target.artifact_type)?)
                .bind(proposal.traceability.target.artifact_id.as_str())
                .bind(serde_json::to_string(&proposal.traceability)?)
                .bind(serde_name(&proposal.promotion_state)?)
                .bind(proposal.duplicate_of.as_ref().map(provenance_core::StableId::as_str))
                .bind(proposal.superseded_by.as_ref().map(provenance_core::StableId::as_str))
                .execute(&mut *tx)
                .await?;
            records_loaded += 1;
        }
        for decision in store.list_promotion_decisions(&scope.id)? {
            sqlx::query("INSERT INTO promotion_decisions (scope_id, id, proposal_id, decision, rationale, actor, canonical_artifact) VALUES (?, ?, ?, ?, ?, ?, ?)")
                .bind(decision.scope_id.as_str())
                .bind(decision.id.as_str())
                .bind(decision.proposal_id.as_str())
                .bind(serde_name(&decision.decision)?)
                .bind(&decision.rationale)
                .bind(serde_json::to_string(&decision.actor)?)
                .bind(decision.canonical_artifact.as_ref().map(serde_json::to_string).transpose()?)
                .execute(&mut *tx)
                .await?;
            records_loaded += 1;
        }
    }
    for edge in store.list_edges()? {
        sqlx::query("INSERT INTO edges (scope_id, id, edge_type, from_type, from_id, to_type, to_id) VALUES (?, ?, ?, ?, ?, ?, ?)")
            .bind(edge.scope_id.as_str())
            .bind(edge.id.as_str())
            .bind(serde_name(&edge.edge_type)?)
            .bind(serde_name(&edge.from_type)?)
            .bind(edge.from_id.as_str())
            .bind(serde_name(&edge.to_type)?)
            .bind(edge.to_id.as_str())
            .execute(&mut *tx)
            .await?;
        records_loaded += 1;
    }
    tx.commit().await?;
    Ok(MaterializeReport {
        records_loaded,
        migrations_applied,
    })
}

pub(crate) fn serde_name<T: serde::Serialize>(value: &T) -> anyhow::Result<String> {
    Ok(serde_json::to_value(value)?.as_str().unwrap().to_string())
}

#[cfg(test)]
mod report_tests {
    use super::*;
    use crate::state_store::{
        AddSourceReferenceInput, CreateDomainInput, CreateRequirementInput, CreateResolutionInput,
        CreateRuleInput, CreateSourceInput,
    };
    use provenance_core::{
        Manifest, NodeType, RepoPathPrefix, RequirementStatus, ResolutionInput,
        ResolutionInputType, ResolutionStatus, RuleSeverity, RuleStatus, ScopeId, SourceType,
        StableId,
    };

    #[allow(clippy::too_many_lines)]
    fn seeded_layout() -> (tempfile::TempDir, ProvenanceLayout, ScopeId) {
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
        let store = StateStore::new(layout.clone());
        store
            .create_domain(CreateDomainInput {
                scope_id: scope.clone(),
                id: StableId::new("domain_payroll").unwrap(),
                name: "Payroll".into(),
                description: None,
                color: None,
            })
            .unwrap();
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
                id: StableId::new("req_schads_overtime").unwrap(),
                statement: "Overtime".into(),
                description: None,
                status: RequirementStatus::Active,
                domain_id: Some(StableId::new("domain_payroll").unwrap()),
                origin_thread: None,
                origin_message: None,
            })
            .unwrap();
        store
            .add_source_reference(AddSourceReferenceInput {
                scope_id: scope.clone(),
                source_id: StableId::new("source_schads").unwrap(),
                requirement_id: StableId::new("req_schads_overtime").unwrap(),
                clause: None,
            })
            .unwrap();
        store
            .create_resolution(CreateResolutionInput {
                scope_id: scope.clone(),
                id: StableId::new("res_schads_overtime").unwrap(),
                title: "Overtime interpretation".into(),
                requirement_id: Some(StableId::new("req_schads_overtime").unwrap()),
                position: "Use award threshold".into(),
                rationale: "Matches source clause".into(),
                status: ResolutionStatus::Proposed,
                context: None,
                enforcement: None,
                confidence: None,
                inputs: Vec::new(),
                made_by: None,
                approved_by: None,
                approved_at: None,
                superseded_by: None,
                origin_thread: None,
                origin_message: None,
            })
            .unwrap();
        store
            .create_rule(CreateRuleInput {
                scope_id: scope.clone(),
                id: StableId::new("rule_schads_pay_001").unwrap(),
                rule_code: "SCHADS-PAY-001".into(),
                name: None,
                description: None,
                requirement_id: Some(StableId::new("req_schads_overtime").unwrap()),
                resolution_id: Some(StableId::new("res_schads_overtime").unwrap()),
                statement: "Pay overtime after the threshold".into(),
                status: RuleStatus::Active,
                severity: RuleSeverity::High,
                rule_type: None,
                modality: None,
                confidence: None,
                extraction_method: None,
                source_document: None,
                source_section: None,
                origin_thread: None,
                origin_message: None,
            })
            .unwrap();
        (dir, layout, scope)
    }

    #[test]
    fn impact_reports_hop_distance_and_direction() {
        let (_dir, layout, scope) = seeded_layout();
        let impact = analyze_impact(
            &layout,
            &scope,
            NodeType::Source,
            &StableId::new("source_schads").unwrap(),
            ImpactOptions {
                max_hops: 3,
                follow_indirect: true,
            },
        )
        .unwrap();
        let rule = impact
            .nodes
            .iter()
            .find(|node| node.id == "rule_schads_pay_001")
            .unwrap();
        assert_eq!(rule.hop_distance, 2);
        assert_eq!(rule.direction, ImpactDirection::Downstream);
    }

    #[test]
    fn stale_report_is_empty_for_unapproved_fixture() {
        let (_dir, layout, scope) = seeded_layout();
        assert!(find_stale(&layout, &scope).unwrap().is_empty());
    }

    #[test]
    fn health_counts_rules_with_complete_traceability() {
        let (_dir, layout, scope) = seeded_layout();
        let health = coverage_health(&layout, &scope).unwrap();
        assert_eq!(health.rules.total, 1);
        assert_eq!(health.rules.with_complete_traceability, 1);
        assert_eq!(health.gaps.total, 0);
    }

    #[test]
    fn gaps_flag_requirements_without_domain_id_but_not_requirements_with_one() {
        let (_dir, layout, scope) = seeded_layout();
        let store = StateStore::new(layout.clone());
        store
            .create_requirement(CreateRequirementInput {
                scope_id: scope.clone(),
                id: StableId::new("req_missing_domain").unwrap(),
                statement: "Rostering rules need a domain".into(),
                description: None,
                status: RequirementStatus::Active,
                domain_id: None,
                origin_thread: None,
                origin_message: None,
            })
            .unwrap();

        let gaps = find_gaps(&layout, &scope).unwrap();
        assert!(gaps.iter().any(|gap| {
            gap.requirement_id == "req_missing_domain" && gap.reason.contains("domain_id")
        }));
        assert!(!gaps.iter().any(|gap| {
            gap.requirement_id == "req_schads_overtime" && gap.reason.contains("domain_id")
        }));
    }

    #[tokio::test]
    async fn materialize_state_caches_fog_resolution_method_and_claim_state() {
        let (_dir, layout, scope) = seeded_layout();
        let store = StateStore::new(layout.clone());
        store
            .set_requirement_fog(
                &scope,
                &StableId::new("req_schads_overtime").unwrap(),
                Some("sleepover rules; something about broken shifts".into()),
            )
            .unwrap();
        store
            .create_topic(crate::state_store::CreateTopicInput {
                scope_id: scope.clone(),
                id: StableId::new("topic_overtime").unwrap(),
                requirement_id: StableId::new("req_schads_overtime").unwrap(),
                title: "Overtime eligibility".into(),
                status: provenance_core::TopicStatus::Open,
                links: Vec::new(),
            })
            .unwrap();
        store
            .create_question(crate::state_store::CreateQuestionInput {
                scope_id: scope.clone(),
                id: StableId::new("question_threshold").unwrap(),
                topic_id: StableId::new("topic_overtime").unwrap(),
                question: "Which threshold applies?".into(),
                resolution_method: provenance_core::ResolutionMethod::Verify,
                status: provenance_core::QuestionStatus::Open,
                answer: None,
                links: Vec::new(),
                resolution_id: None,
            })
            .unwrap();
        store
            .claim_topic(
                &scope,
                &StableId::new("topic_overtime").unwrap(),
                "agent-one",
            )
            .unwrap();
        store
            .claim_question(
                &scope,
                &StableId::new("question_threshold").unwrap(),
                "agent-two",
            )
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
        let store = StateStore::new(layout.clone());
        store
            .create_source(CreateSourceInput {
                scope_id: scope.clone(),
                id: StableId::new("source_sah").unwrap(),
                name: "Support at Home".into(),
                source_type: SourceType::Legislation,
                url: Some("https://example.test/sah".into()),
                reference: Some("Department guidance".into()),
                commit_pin: None,
                effective_date: Some(1_714_521_600_000),
                review_date: Some(1_717_200_000_000),
                superseded_by: Some(StableId::new("source_sah_2025").unwrap()),
                origin_thread: None,
                origin_message: None,
            })
            .unwrap();
        store
            .create_resolution(CreateResolutionInput {
                scope_id: scope.clone(),
                id: StableId::new("res_sah").unwrap(),
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
                superseded_by: Some(StableId::new("res_sah_2025").unwrap()),
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
        let resolution: (String, Option<String>, Option<String>, Option<i64>, Option<String>) =
            sqlx::query_as(
                "SELECT inputs, made_by, approved_by, approved_at, superseded_by FROM resolutions WHERE id = ?",
            )
            .bind("res_sah")
            .fetch_one(&pool)
            .await
            .unwrap();

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

        let sources_path = crate::shards::sources_path(&layout, &scope);
        std::fs::create_dir_all(sources_path.parent().unwrap()).unwrap();
        std::fs::write(
            &sources_path,
            r#"{"schema_version":1,"scope_id":"default","id":"source_codebase","name":"Example API","source_type":"project_artifact","commit_pin":"5e1f2a9c4b6d8e0f1234567890abcdef12345678"}
"#,
        )
        .unwrap();

        let contributions_path = crate::shards::contributions_path(&layout, &scope);
        std::fs::create_dir_all(contributions_path.parent().unwrap()).unwrap();
        std::fs::write(
            &contributions_path,
            r#"{"schema_version":1,"scope_id":"default","id":"contrib_reviewer_001","target":{"artifact_type":"requirement","artifact_id":"req_overtime"},"participant_slot":"reviewer","stance":"support","strongest_finding":"Supported by code evidence.","evidence_references":[],"material_claims":[{"claim_id":"claim_overtime_threshold","statement":"Overtime starts after the award threshold.","evidence_type":"artifact","evidence_reference_ids":[],"confidence":0.87}],"risks":[],"objections":[],"challenges":[],"suggested_artifact_changes":[],"unsupported_recommendations":[],"uncertainty":{"level":"low","rationale":"Direct code evidence."},"open_questions":[]}
"#,
        )
        .unwrap();

        let proposals_path = crate::shards::proposal_cards_path(&layout, &scope);
        std::fs::create_dir_all(proposals_path.parent().unwrap()).unwrap();
        std::fs::write(
            &proposals_path,
            r#"{"schema_version":1,"scope_id":"default","id":"proposal_overtime_traceability","proposal_key":"req-overtime-traceability","proposal_type":"requirement_candidate","title":"Clarify overtime traceability","summary":"Add source-backed threshold language.","confidence":0.83,"traceability":{"target":{"artifact_type":"requirement","artifact_id":"req_overtime"},"source_ids":["source_codebase"],"evidence_references":[],"supporting_claim_ids":["claim_overtime_threshold"]},"promotion_state":"proposed"}
"#,
        )
        .unwrap();

        materialize_state(&layout).await.unwrap();
        let pool = open_cache(&layout).await.unwrap();
        let commit_pin: Option<String> =
            sqlx::query_scalar("SELECT commit_pin FROM sources WHERE id = ?")
                .bind("source_codebase")
                .fetch_one(&pool)
                .await
                .unwrap();
        let proposal_confidence: Option<f64> =
            sqlx::query_scalar("SELECT confidence FROM proposal_cards WHERE id = ?")
                .bind("proposal_overtime_traceability")
                .fetch_one(&pool)
                .await
                .unwrap();
        let contribution_payload: String =
            sqlx::query_scalar("SELECT payload FROM contributions WHERE id = ?")
                .bind("contrib_reviewer_001")
                .fetch_one(&pool)
                .await
                .unwrap();

        assert_eq!(
            commit_pin.as_deref(),
            Some("5e1f2a9c4b6d8e0f1234567890abcdef12345678")
        );
        assert_eq!(proposal_confidence, Some(0.83));
        assert!(contribution_payload.contains(r#""confidence":0.87"#));
    }
}
