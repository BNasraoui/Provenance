use sqlx::{Executor, SqlitePool};

pub const INITIAL_MIGRATION_ID: &str = "001";
pub const SOURCE_REQUIREMENT_MIGRATION_ID: &str = "002";
pub const RESOLUTIONS_RULES_MIGRATION_ID: &str = "003";
pub const THREADS_MESSAGES_MIGRATION_ID: &str = "004";
pub const REPORT_INDEXES_MIGRATION_ID: &str = "005";
pub const IDEATION_OUTPUTS_MIGRATION_ID: &str = "006";
const INITIAL_SQL: &str = include_str!("../migrations/001_initial_cache.sql");
const SOURCE_REQUIREMENT_SQL: &str =
    include_str!("../migrations/002_sources_requirements_edges.sql");
const RESOLUTIONS_RULES_SQL: &str = include_str!("../migrations/003_resolutions_rules.sql");
const THREADS_MESSAGES_SQL: &str = include_str!("../migrations/004_threads_messages.sql");
const REPORT_INDEXES_SQL: &str = include_str!("../migrations/005_report_indexes.sql");
const IDEATION_OUTPUTS_SQL: &str = include_str!("../migrations/006_ideation_outputs.sql");

pub async fn run_migrations(pool: &SqlitePool) -> anyhow::Result<Vec<String>> {
    pool.execute("CREATE TABLE IF NOT EXISTS _schema_migrations (id TEXT PRIMARY KEY, applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP)").await?;
    let mut tx = pool.begin().await?;
    let mut applied = Vec::new();
    for (id, sql) in [
        (INITIAL_MIGRATION_ID, INITIAL_SQL),
        (SOURCE_REQUIREMENT_MIGRATION_ID, SOURCE_REQUIREMENT_SQL),
        (RESOLUTIONS_RULES_MIGRATION_ID, RESOLUTIONS_RULES_SQL),
        (THREADS_MESSAGES_MIGRATION_ID, THREADS_MESSAGES_SQL),
        (REPORT_INDEXES_MIGRATION_ID, REPORT_INDEXES_SQL),
        (IDEATION_OUTPUTS_MIGRATION_ID, IDEATION_OUTPUTS_SQL),
    ] {
        let already_applied: Option<String> =
            sqlx::query_scalar("SELECT id FROM _schema_migrations WHERE id = ?")
                .bind(id)
                .fetch_optional(&mut *tx)
                .await?;
        if already_applied.is_none() {
            for statement in sql.split(';').map(str::trim).filter(|s| !s.is_empty()) {
                tx.execute(statement).await?;
            }
            sqlx::query("INSERT INTO _schema_migrations (id) VALUES (?)")
                .bind(id)
                .execute(&mut *tx)
                .await?;
            applied.push(id.to_string());
        }
    }
    tx.commit().await?;
    Ok(applied)
}

pub async fn applied_migrations(pool: &SqlitePool) -> anyhow::Result<Vec<String>> {
    Ok(
        sqlx::query_scalar("SELECT id FROM _schema_migrations ORDER BY id")
            .fetch_all(pool)
            .await?,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn migrations_record_initial_cache_schema_once() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        assert_eq!(
            run_migrations(&pool).await.unwrap(),
            vec!["001", "002", "003", "004", "005", "006"]
        );
        assert!(run_migrations(&pool).await.unwrap().is_empty());
        assert_eq!(
            applied_migrations(&pool).await.unwrap(),
            vec!["001", "002", "003", "004", "005", "006"]
        );
    }
}
