mod collaboration_records;
mod graph_records;

use super::{open_cache, MaterializeReport};
use crate::{layout::ProvenanceLayout, migrations, state_store::StateStore};
use sqlx::{Sqlite, Transaction};

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

pub async fn materialize_state(layout: &ProvenanceLayout) -> anyhow::Result<MaterializeReport> {
    let pool = open_cache(layout).await?;
    let migrations_applied = migrations::run_migrations(&pool).await?;
    let store = StateStore::new(layout.clone());
    let manifest = store.manifest()?;
    let manifest_scopes = manifest
        .scopes
        .iter()
        .map(|scope| &scope.id)
        .collect::<std::collections::BTreeSet<_>>();
    for edge in store.list_edges()? {
        anyhow::ensure!(
            manifest_scopes.contains(&edge.scope_id),
            "edge scope_id must name a manifest scope"
        );
    }
    for scope in &manifest.scopes {
        validate_scope_ownership(&store, &scope.id)?;
        store.validate_ideation_scope(&scope.id)?;
    }
    let mut tx = pool.begin().await?;
    clear_cache(&mut tx).await?;

    let mut records_loaded = 0;
    for scope in &manifest.scopes {
        records_loaded += graph_records::load_scope(&mut tx, &store, &scope.id).await?;
        records_loaded += collaboration_records::load_scope(&mut tx, &store, &scope.id).await?;
    }
    records_loaded += graph_records::load_edges(&mut tx, &store).await?;
    tx.commit().await?;

    Ok(MaterializeReport {
        records_loaded,
        migrations_applied,
    })
}

fn validate_scope_ownership(
    store: &StateStore,
    expected: &provenance_core::ScopeId,
) -> anyhow::Result<()> {
    macro_rules! owned {
        ($kind:literal, $records:expr) => {
            for record in $records {
                anyhow::ensure!(
                    record.scope_id == *expected,
                    "{} scope_id must match containing scope",
                    $kind
                );
            }
        };
    }
    owned!("source", store.list_sources(expected)?);
    owned!("domain", store.list_domains(expected)?);
    owned!("requirement", store.list_requirements(expected)?);
    owned!("boundary", store.list_boundaries(expected)?);
    owned!("topic", store.list_topics(expected)?);
    owned!("question", store.list_questions(expected)?);
    owned!("resolution", store.list_resolutions(expected)?);
    owned!("rule", store.list_rules(expected)?);
    owned!("service", store.list_services(expected)?);
    owned!("service binding", store.list_service_bindings(expected)?);
    owned!("thread", store.list_threads(expected)?);
    owned!("message", store.list_messages(expected)?);
    Ok(())
}

async fn clear_cache(tx: &mut Transaction<'_, Sqlite>) -> anyhow::Result<()> {
    for table in [
        "sources",
        "domains",
        "requirements",
        "boundaries",
        "topics",
        "questions",
        "edges",
        "resolutions",
        "rules",
        "services",
        "service_bindings",
        "messages",
        "threads",
        "contributions",
        "synthesis_packets",
        "proposal_cards",
        "assertion_records",
        "promotion_decisions",
    ] {
        sqlx::query(&format!("DELETE FROM {table}"))
            .execute(&mut **tx)
            .await?;
    }
    Ok(())
}
