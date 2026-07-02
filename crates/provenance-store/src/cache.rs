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
    sqlx::query("DELETE FROM requirements")
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM edges").execute(&mut *tx).await?;
    sqlx::query("DELETE FROM resolutions")
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM rules").execute(&mut *tx).await?;
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
                "INSERT INTO sources (scope_id, id, name, source_type, url) VALUES (?, ?, ?, ?, ?)",
            )
            .bind(source.scope_id.as_str())
            .bind(source.id.as_str())
            .bind(source.name)
            .bind(serde_name(&source.source_type)?)
            .bind(source.url)
            .execute(&mut *tx)
            .await?;
            records_loaded += 1;
        }
        for requirement in store.list_requirements(&scope.id)? {
            sqlx::query(
                "INSERT INTO requirements (scope_id, id, statement, status) VALUES (?, ?, ?, ?)",
            )
            .bind(requirement.scope_id.as_str())
            .bind(requirement.id.as_str())
            .bind(requirement.statement)
            .bind(serde_name(&requirement.status)?)
            .execute(&mut *tx)
            .await?;
            records_loaded += 1;
        }
        for resolution in store.list_resolutions(&scope.id)? {
            sqlx::query("INSERT INTO resolutions (scope_id, id, title, position, rationale, status, review_on, review_triggers) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
                .bind(resolution.scope_id.as_str())
                .bind(resolution.id.as_str())
                .bind(resolution.title)
                .bind(resolution.position)
                .bind(resolution.rationale)
                .bind(serde_name(&resolution.status)?)
                .bind(resolution.review_on)
                .bind(resolution.review_triggers.to_string())
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
            sqlx::query("INSERT INTO proposal_cards (scope_id, id, proposal_key, proposal_type, title, summary, target_type, target_id, traceability, promotion_state, duplicate_of, superseded_by) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
                .bind(proposal.scope_id.as_str())
                .bind(proposal.id.as_str())
                .bind(&proposal.proposal_key)
                .bind(serde_name(&proposal.proposal_type)?)
                .bind(&proposal.title)
                .bind(&proposal.summary)
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
        AddSourceReferenceInput, CreateRequirementInput, CreateResolutionInput, CreateRuleInput,
        CreateSourceInput,
    };
    use provenance_core::{
        Manifest, NodeType, RepoPathPrefix, RequirementStatus, ResolutionStatus, RuleSeverity,
        RuleStatus, ScopeId, SourceType, StableId,
    };

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
            .create_source(CreateSourceInput {
                scope_id: scope.clone(),
                id: StableId::new("source_schads").unwrap(),
                name: "SCHADS Award".into(),
                source_type: SourceType::Policy,
                url: None,
                reference: None,
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
}
