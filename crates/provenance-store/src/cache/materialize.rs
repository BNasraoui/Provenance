mod collaboration_records;
mod graph_records;

use super::{open_cache, MaterializeReport};
use crate::{layout::ProvenanceLayout, migrations, publication, state_store::StateStore};
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
    let snapshot = publication::snapshot_state(layout)?;
    let store = StateStore::new(snapshot.layout().clone());
    let manifest = store.manifest()?;
    for scope in &manifest.scopes {
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
        "dispositions",
    ] {
        sqlx::query(&format!("DELETE FROM {table}"))
            .execute(&mut **tx)
            .await?;
    }
    Ok(())
}
