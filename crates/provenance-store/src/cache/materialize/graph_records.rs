use crate::{cache::serde_name, state_store::ScopeSnapshot};
use sqlx::{Sqlite, Transaction};

pub(super) async fn load_scope(
    tx: &mut Transaction<'_, Sqlite>,
    snapshot: &ScopeSnapshot,
) -> anyhow::Result<u64> {
    let mut loaded = load_requirement_records(tx, snapshot).await?;
    loaded += load_decision_records(tx, snapshot).await?;
    loaded += load_service_records(tx, snapshot).await?;
    Ok(loaded)
}

async fn load_requirement_records(
    tx: &mut Transaction<'_, Sqlite>,
    snapshot: &ScopeSnapshot,
) -> anyhow::Result<u64> {
    let mut loaded = 0;
    for source in snapshot.sources.clone() {
        sqlx::query("INSERT INTO sources (scope_id, id, name, source_type, url, reference, commit_pin, effective_date, review_date, superseded_by) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(source.scope_id.as_str()).bind(source.id.as_str()).bind(source.name)
            .bind(serde_name(&source.source_type)?).bind(source.url).bind(source.reference)
            .bind(source.commit_pin).bind(source.effective_date).bind(source.review_date)
            .bind(source.superseded_by.as_ref().map(provenance_core::StableId::as_str))
            .execute(&mut **tx).await?;
        loaded += 1;
    }
    for requirement in snapshot.requirements.clone() {
        sqlx::query("INSERT INTO requirements (scope_id, id, statement, status, domain_id, fog) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(requirement.scope_id.as_str()).bind(requirement.id.as_str())
            .bind(requirement.statement).bind(serde_name(&requirement.status)?)
            .bind(requirement.domain_id.as_ref().map(provenance_core::StableId::as_str))
            .bind(requirement.fog).execute(&mut **tx).await?;
        loaded += 1;
    }
    for domain in snapshot.domains.clone() {
        sqlx::query(
            "INSERT INTO domains (scope_id, id, name, description, color) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(domain.scope_id.as_str())
        .bind(domain.id.as_str())
        .bind(domain.name)
        .bind(domain.description)
        .bind(domain.color)
        .execute(&mut **tx)
        .await?;
        loaded += 1;
    }
    for boundary in snapshot.boundaries.clone() {
        let source_id = boundary
            .source_ref
            .as_ref()
            .map(|reference| reference.source_id.as_str());
        let source_clause = boundary
            .source_ref
            .as_ref()
            .and_then(|reference| reference.clause.as_deref());
        sqlx::query("INSERT INTO boundaries (scope_id, id, requirement_id, statement, source_id, source_clause) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(boundary.scope_id.as_str()).bind(boundary.id.as_str())
            .bind(boundary.requirement_id.as_str()).bind(boundary.statement)
            .bind(source_id).bind(source_clause).execute(&mut **tx).await?;
        loaded += 1;
    }
    Ok(loaded)
}

async fn load_decision_records(
    tx: &mut Transaction<'_, Sqlite>,
    snapshot: &ScopeSnapshot,
) -> anyhow::Result<u64> {
    let mut loaded = 0;
    for topic in snapshot.topics.clone() {
        sqlx::query("INSERT INTO topics (scope_id, id, requirement_id, title, status, claimed_by, claimed_at, links) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(topic.scope_id.as_str()).bind(topic.id.as_str()).bind(topic.requirement_id.as_str())
            .bind(topic.title).bind(serde_name(&topic.status)?).bind(topic.claimed_by)
            .bind(topic.claimed_at).bind(serde_json::to_string(&topic.links)?)
            .execute(&mut **tx).await?;
        loaded += 1;
    }
    for question in snapshot.questions.clone() {
        sqlx::query("INSERT INTO questions (scope_id, id, topic_id, requirement_id, question, resolution_method, status, claimed_by, claimed_at, answer, links, resolution_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(question.scope_id.as_str()).bind(question.id.as_str()).bind(question.topic_id.as_str())
            .bind(question.requirement_id.as_str()).bind(question.question)
            .bind(serde_name(&question.resolution_method)?).bind(serde_name(&question.status)?)
            .bind(question.claimed_by).bind(question.claimed_at).bind(question.answer)
            .bind(serde_json::to_string(&question.links)?)
            .bind(question.resolution_id.as_ref().map(provenance_core::StableId::as_str))
            .execute(&mut **tx).await?;
        loaded += 1;
    }
    for resolution in snapshot.resolutions.clone() {
        sqlx::query("INSERT INTO resolutions (scope_id, id, title, position, rationale, status, review_on, review_triggers, context, enforcement, confidence, inputs, made_by, approved_by, approved_at, superseded_by) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(resolution.scope_id.as_str()).bind(resolution.id.as_str()).bind(resolution.title)
            .bind(resolution.position).bind(resolution.rationale).bind(serde_name(&resolution.status)?)
            .bind(resolution.review_on).bind(resolution.review_triggers.to_string())
            .bind(resolution.context).bind(resolution.enforcement).bind(resolution.confidence)
            .bind(serde_json::to_string(&resolution.inputs)?).bind(resolution.made_by)
            .bind(resolution.approved_by).bind(resolution.approved_at)
            .bind(resolution.superseded_by.as_ref().map(provenance_core::StableId::as_str))
            .execute(&mut **tx).await?;
        loaded += 1;
    }
    for rule in snapshot.rules.clone() {
        sqlx::query("INSERT INTO rules (scope_id, id, rule_code, statement, status, severity, expression, inputs) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(rule.scope_id.as_str()).bind(rule.id.as_str()).bind(rule.rule_code)
            .bind(rule.statement).bind(serde_name(&rule.status)?).bind(serde_name(&rule.severity)?)
            .bind(rule.expression.to_string()).bind(rule.inputs.to_string())
            .execute(&mut **tx).await?;
        loaded += 1;
    }
    Ok(loaded)
}

async fn load_service_records(
    tx: &mut Transaction<'_, Sqlite>,
    snapshot: &ScopeSnapshot,
) -> anyhow::Result<u64> {
    let mut loaded = 0;
    for service in snapshot.services.clone() {
        sqlx::query("INSERT INTO services (scope_id, id, name, description, owner, repository, environment, tier, external_id, status) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(service.scope_id.as_str()).bind(service.id.as_str()).bind(service.name)
            .bind(service.description).bind(service.owner).bind(service.repository)
            .bind(service.environment.as_ref().map(serde_name).transpose()?)
            .bind(service.tier.as_ref().map(serde_name).transpose()?).bind(service.external_id)
            .bind(serde_name(&service.status)?).execute(&mut **tx).await?;
        loaded += 1;
    }
    for binding in snapshot.service_bindings.clone() {
        sqlx::query("INSERT INTO service_bindings (scope_id, id, rule_id, service_id, binding_type) VALUES (?, ?, ?, ?, ?)")
            .bind(binding.scope_id.as_str()).bind(binding.id.as_str()).bind(binding.rule_id.as_str())
            .bind(binding.service_id.as_str()).bind(serde_name(&binding.binding_type)?)
            .execute(&mut **tx).await?;
        loaded += 1;
    }
    Ok(loaded)
}

pub(super) async fn load_edges(
    tx: &mut Transaction<'_, Sqlite>,
    edges: &[provenance_core::Edge],
) -> anyhow::Result<u64> {
    let mut loaded = 0;
    for edge in edges {
        sqlx::query("INSERT INTO edges (scope_id, id, edge_type, from_type, from_id, to_type, to_id) VALUES (?, ?, ?, ?, ?, ?, ?)")
            .bind(edge.scope_id.as_str()).bind(edge.id.as_str()).bind(serde_name(&edge.edge_type)?)
            .bind(serde_name(&edge.from_type)?).bind(edge.from_id.as_str())
            .bind(serde_name(&edge.to_type)?).bind(edge.to_id.as_str())
            .execute(&mut **tx).await?;
        loaded += 1;
    }
    Ok(loaded)
}
