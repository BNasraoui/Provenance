use provenance_core::{
    ScopeId, StableId, VerificationRun, VerificationRunStatus, SUPPORTED_SCHEMA_VERSION,
};

use super::{BeginVerificationInput, CompleteVerificationInput, StateStore};

impl StateStore {
    pub fn begin_verification(
        &self,
        scope_id: ScopeId,
        input: BeginVerificationInput,
    ) -> anyhow::Result<VerificationRun> {
        anyhow::ensure!(
            !input.declared_by.trim().is_empty(),
            "declared_by must not be empty"
        );
        anyhow::ensure!(!input.method.trim().is_empty(), "method must not be empty");
        let rule_id = StableId::new(input.rule)?;
        anyhow::ensure!(
            self.list_rules(&scope_id)?
                .iter()
                .any(|rule| rule.id == rule_id),
            "rule `{}` does not exist",
            rule_id.as_str()
        );
        let started_at = now_millis()?;
        let path = self.layout.verification_runs_path(&scope_id);
        let lock_path = self.layout.verification_runs_lock_path(&scope_id);
        crate::jsonl::mutate_jsonl_locked(
            &path,
            &lock_path,
            |records: &mut Vec<VerificationRun>| {
                let id = next_run_id(records, started_at)?;
                let run = VerificationRun {
                    schema_version: SUPPORTED_SCHEMA_VERSION,
                    scope_id,
                    id,
                    rule_id,
                    method: input.method,
                    declared_by: input.declared_by,
                    file: input.file,
                    symbol: input.symbol,
                    status: VerificationRunStatus::Running,
                    started_at,
                    completed_at: None,
                    error: None,
                };
                records.push(run.clone());
                records.sort_by(|left, right| {
                    left.started_at
                        .cmp(&right.started_at)
                        .then(left.id.as_str().cmp(right.id.as_str()))
                });
                Ok(run)
            },
        )
    }

    pub fn complete_verification(
        &self,
        scope_id: &ScopeId,
        input: CompleteVerificationInput,
    ) -> anyhow::Result<VerificationRun> {
        let run_id = StableId::new(input.run)?;
        let status = VerificationRunStatus::parse_completion(&input.status)?;
        anyhow::ensure!(
            status == VerificationRunStatus::Failed || input.error.is_none(),
            "a passed verification cannot carry an error"
        );
        let completed_at = now_millis()?;
        let path = self.layout.verification_runs_path(scope_id);
        let lock_path = self.layout.verification_runs_lock_path(scope_id);
        crate::jsonl::mutate_jsonl_locked(
            &path,
            &lock_path,
            |records: &mut Vec<VerificationRun>| {
                let run = records
                    .iter_mut()
                    .find(|run| run.scope_id == *scope_id && run.id == run_id)
                    .ok_or_else(|| {
                        anyhow::anyhow!("verification run `{}` does not exist", run_id.as_str())
                    })?;
                anyhow::ensure!(
                    run.status == VerificationRunStatus::Running,
                    "verification run `{}` is already complete",
                    run_id.as_str()
                );
                run.status = status;
                run.completed_at = Some(completed_at.max(run.started_at));
                run.error = input.error;
                Ok(run.clone())
            },
        )
    }

    pub fn list_verification_runs(
        &self,
        scope_id: &ScopeId,
    ) -> anyhow::Result<Vec<VerificationRun>> {
        let path = self.layout.verification_runs_path(scope_id);
        let lock_path = self.layout.verification_runs_lock_path(scope_id);
        crate::jsonl::with_advisory_lock(&lock_path, || {
            if !path.exists() {
                return Ok(Vec::new());
            }
            std::fs::read_to_string(path)?
                .lines()
                .map(|line| serde_json::from_str(line).map_err(Into::into))
                .collect()
        })
    }
}

fn next_run_id(records: &[VerificationRun], started_at: i64) -> anyhow::Result<StableId> {
    let base = format!("verification_{started_at}");
    let mut candidate = base.clone();
    let mut suffix = 2_u64;
    while records.iter().any(|run| run.id.as_str() == candidate) {
        candidate = format!("{base}_{suffix}");
        suffix = suffix
            .checked_add(1)
            .ok_or_else(|| anyhow::anyhow!("verification run id suffix overflow"))?;
    }
    StableId::new(candidate)
}

fn now_millis() -> anyhow::Result<i64> {
    let duration = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?;
    i64::try_from(duration.as_millis()).map_err(Into::into)
}
