use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MergeConflictKind {
    DivergentEdit,
    DeleteModify,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct MergeConflict {
    pub kind: MergeConflictKind,
    pub record_id: String,
    pub ours: Option<Value>,
    pub theirs: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum MergeOutcome<T> {
    Clean {
        records: T,
    },
    Conflicted {
        conflicts: Vec<MergeConflict>,
        partial: T,
    },
}

impl<T> MergeOutcome<T> {
    pub fn unwrap_clean(self) -> T {
        match self {
            Self::Clean { records } => records,
            Self::Conflicted { .. } => panic!("expected clean merge"),
        }
    }

    pub fn unwrap_conflicts(self) -> Vec<MergeConflict> {
        match self {
            Self::Clean { .. } => panic!("expected conflicted merge"),
            Self::Conflicted { conflicts, .. } => conflicts,
        }
    }
}

pub type CanonicalRecord = Value;

pub fn merge_records(
    base: &[CanonicalRecord],
    ours: &[CanonicalRecord],
    theirs: &[CanonicalRecord],
) -> anyhow::Result<MergeOutcome<Vec<CanonicalRecord>>> {
    let base = index_by_id(base)?;
    let ours = index_by_id(ours)?;
    let theirs = index_by_id(theirs)?;
    let mut ids = BTreeSet::new();
    ids.extend(base.keys().cloned());
    ids.extend(ours.keys().cloned());
    ids.extend(theirs.keys().cloned());

    let mut merged = Vec::new();
    let mut conflicts = Vec::new();
    for id in ids {
        match (base.get(&id), ours.get(&id), theirs.get(&id)) {
            (None, Some(ours), None) => merged.push(ours.clone()),
            (None, None, Some(theirs)) => merged.push(theirs.clone()),
            (None, Some(ours), Some(theirs)) if ours == theirs => merged.push(ours.clone()),
            (Some(_), None, None) => {}
            (Some(base), Some(ours), None) if ours == base => {}
            (Some(_), Some(ours), None) => {
                merged.push(ours.clone());
                conflicts.push(MergeConflict {
                    kind: MergeConflictKind::DeleteModify,
                    record_id: id,
                    ours: Some(ours.clone()),
                    theirs: None,
                });
            }
            (Some(base), None, Some(theirs)) if theirs == base => {}
            (Some(_), None, Some(theirs)) => {
                merged.push(theirs.clone());
                conflicts.push(MergeConflict {
                    kind: MergeConflictKind::DeleteModify,
                    record_id: id,
                    ours: None,
                    theirs: Some(theirs.clone()),
                });
            }
            (Some(_), Some(ours), Some(theirs)) if ours == theirs => merged.push(ours.clone()),
            (Some(base), Some(ours), Some(theirs)) if ours == base => merged.push(theirs.clone()),
            (Some(base), Some(ours), Some(theirs)) if theirs == base => merged.push(ours.clone()),
            (None | Some(_), Some(ours), Some(theirs)) => {
                merged.push(ours.clone());
                conflicts.push(MergeConflict {
                    kind: MergeConflictKind::DivergentEdit,
                    record_id: id,
                    ours: Some(ours.clone()),
                    theirs: Some(theirs.clone()),
                });
            }
            (None, None, None) => unreachable!(),
        }
    }

    if conflicts.is_empty() {
        Ok(MergeOutcome::Clean { records: merged })
    } else {
        Ok(MergeOutcome::Conflicted {
            conflicts,
            partial: merged,
        })
    }
}

pub fn read_jsonl_records(path: &camino::Utf8Path) -> anyhow::Result<Vec<CanonicalRecord>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    std::fs::read_to_string(path)?
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| Ok(serde_json::from_str(line)?))
        .collect()
}

fn index_by_id(records: &[CanonicalRecord]) -> anyhow::Result<BTreeMap<String, CanonicalRecord>> {
    let mut indexed = BTreeMap::new();
    for record in records {
        let id = record
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("record is missing string id"))?;
        anyhow::ensure!(!indexed.contains_key(id), "duplicate record id {id}");
        indexed.insert(id.to_string(), record.clone());
    }
    Ok(indexed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(id: &str, statement: &str) -> Value {
        serde_json::json!({ "id": id, "statement": statement })
    }

    fn ids(records: &[Value]) -> Vec<&str> {
        records
            .iter()
            .map(|record| record.get("id").unwrap().as_str().unwrap())
            .collect()
    }

    #[test]
    fn merge_keeps_one_sided_additions_and_sorts_by_stable_key() {
        let merged = merge_records(&[], &[record("rule_b", "b")], &[record("rule_a", "a")])
            .unwrap()
            .unwrap_clean();

        assert_eq!(ids(&merged), vec!["rule_a", "rule_b"]);
    }

    #[test]
    fn merge_collapses_identical_edits() {
        let base = [record("rule_a", "old")];
        let ours = [record("rule_a", "new")];
        let theirs = [record("rule_a", "new")];

        let merged = merge_records(&base, &ours, &theirs).unwrap().unwrap_clean();

        assert_eq!(merged, vec![record("rule_a", "new")]);
    }

    #[test]
    fn merge_reports_divergent_edits() {
        let base = [record("rule_a", "old")];
        let ours = [record("rule_a", "ours")];
        let theirs = [record("rule_a", "theirs")];

        let conflicts = merge_records(&base, &ours, &theirs)
            .unwrap()
            .unwrap_conflicts();

        assert_eq!(conflicts[0].kind, MergeConflictKind::DivergentEdit);
        assert_eq!(conflicts[0].record_id, "rule_a");
    }

    #[test]
    fn merge_reports_delete_modify_conflict() {
        let base = [record("rule_a", "old")];
        let theirs = [record("rule_a", "new")];

        let conflicts = merge_records(&base, &[], &theirs)
            .unwrap()
            .unwrap_conflicts();

        assert_eq!(conflicts[0].kind, MergeConflictKind::DeleteModify);
        assert_eq!(conflicts[0].record_id, "rule_a");
    }
}
