use super::git::{BlobRead, GitFailure, ResolvedRevision, RevisionReader};
use super::model::{EvidenceSite, VerificationOutcome};
use camino::Utf8Path;

pub struct VerifiedSite<'a> {
    pub site: &'a EvidenceSite,
    pub base: &'a ResolvedRevision,
    pub head: &'a ResolvedRevision,
    pub current_path: &'a str,
    pub outcome: VerificationOutcome,
}

#[allow(clippy::too_many_arguments)]
pub fn group<'a>(
    reader: &dyn RevisionReader,
    repository: &Utf8Path,
    base: &'a ResolvedRevision,
    head: &'a ResolvedRevision,
    old_path: &'a str,
    current_path: &'a str,
    sites: &[&'a EvidenceSite],
) -> Result<Vec<VerifiedSite<'a>>, GitFailure> {
    let old_blob = reader.blob_at(repository, base, old_path)?;
    let current_blob = reader.blob_at(repository, head, current_path)?;
    Ok(sites
        .iter()
        .map(|site| VerifiedSite {
            site,
            base,
            head,
            current_path,
            outcome: classify(site, old_path != current_path, &old_blob, &current_blob),
        })
        .collect())
}

fn classify(
    site: &EvidenceSite,
    renamed: bool,
    old_blob: &BlobRead,
    current_blob: &BlobRead,
) -> VerificationOutcome {
    let BlobRead::Present(old_file) = old_blob else {
        return VerificationOutcome::Unverifiable {
            reason: "recorded evidence path is absent at the pinned commit",
        };
    };
    let Some(recorded_line) = site.line else {
        return VerificationOutcome::Unverifiable {
            reason: "evidence reference has no recorded line",
        };
    };
    let Some(needle) = line_text(old_file, recorded_line) else {
        return VerificationOutcome::Unverifiable {
            reason: "recorded evidence line is absent at the pinned commit",
        };
    };
    let BlobRead::Present(current_file) = current_blob else {
        return VerificationOutcome::Vanished;
    };
    let matches = matching_lines(current_file, needle);
    match matches.as_slice() {
        [] => VerificationOutcome::Vanished,
        [line] if renamed || *line != recorded_line => VerificationOutcome::Moved { line: *line },
        [line] => VerificationOutcome::Verified { line: *line },
        _ => VerificationOutcome::Unverifiable {
            reason: "cited line now has multiple exact matches",
        },
    }
}

fn line_text(file: &str, line: u32) -> Option<&str> {
    let index = usize::try_from(line).ok()?.checked_sub(1)?;
    file.lines()
        .nth(index)
        .filter(|text| !text.trim().is_empty())
}

fn matching_lines(file: &str, needle: &str) -> Vec<u32> {
    file.lines()
        .enumerate()
        .filter(|(_, line)| *line == needle)
        .filter_map(|(index, _)| u32::try_from(index + 1).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::group;
    use crate::handlers::evidence_review::git::{
        BlobRead, CommitTimestampSeconds, GitFailure, PathChange, ResolvedRevision, RevisionReader,
    };
    use crate::handlers::evidence_review::model::{
        EvidenceOwner, EvidenceSite, OwnerKind, RequirementOwnership,
    };
    use camino::{Utf8Path, Utf8PathBuf};
    use provenance_core::StableId;
    use std::cell::Cell;

    struct FailingReader;

    impl RevisionReader for FailingReader {
        fn resolve(
            &self,
            _repository: &Utf8Path,
            _revision: &str,
        ) -> Result<ResolvedRevision, GitFailure> {
            unreachable!()
        }
        fn commit_timestamp(
            &self,
            _repository: &Utf8Path,
            _revision: &ResolvedRevision,
        ) -> Result<CommitTimestampSeconds, GitFailure> {
            unreachable!()
        }
        fn changed_paths(
            &self,
            _repository: &Utf8Path,
            _base: &ResolvedRevision,
            _head: &ResolvedRevision,
        ) -> Result<Vec<PathChange>, GitFailure> {
            unreachable!()
        }
        fn blob_at(
            &self,
            _repository: &Utf8Path,
            revision: &ResolvedRevision,
            _path: &str,
        ) -> Result<BlobRead, GitFailure> {
            if revision.0 == "base" {
                Ok(BlobRead::Present("cited\n".into()))
            } else {
                Err(GitFailure::Command {
                    operation: "show".into(),
                    stderr: "disk failure".into(),
                })
            }
        }
    }

    #[test]
    fn git_failure_is_not_an_evidence_outcome() {
        let site = EvidenceSite {
            owner: EvidenceOwner {
                kind: OwnerKind::Proposal,
                id: StableId::new("proposal").unwrap(),
                title: None,
                ratified: false,
            },
            ownership: RequirementOwnership::TargetRequirement {
                proposal_id: StableId::new("proposal").unwrap(),
                requirement_id: StableId::new("requirement").unwrap(),
            },
            source_id: StableId::new("source").unwrap(),
            repository: Utf8PathBuf::from("."),
            source_revision: Some("base".into()),
            revision: "base".into(),
            reference_id: StableId::new("evidence").unwrap(),
            path: "src/lib.rs".into(),
            line: Some(1),
        };
        let error = group(
            &FailingReader,
            Utf8Path::new("."),
            &ResolvedRevision("base".into()),
            &ResolvedRevision("head".into()),
            "src/lib.rs",
            "src/lib.rs",
            &[&site],
        )
        .err()
        .unwrap();
        assert!(error.to_string().contains("disk failure"));
    }

    struct CountingReader(Cell<usize>);

    impl RevisionReader for CountingReader {
        fn resolve(
            &self,
            _repository: &Utf8Path,
            _revision: &str,
        ) -> Result<ResolvedRevision, GitFailure> {
            unreachable!()
        }
        fn commit_timestamp(
            &self,
            _repository: &Utf8Path,
            _revision: &ResolvedRevision,
        ) -> Result<CommitTimestampSeconds, GitFailure> {
            unreachable!()
        }
        fn changed_paths(
            &self,
            _repository: &Utf8Path,
            _base: &ResolvedRevision,
            _head: &ResolvedRevision,
        ) -> Result<Vec<PathChange>, GitFailure> {
            unreachable!()
        }
        fn blob_at(
            &self,
            _repository: &Utf8Path,
            _revision: &ResolvedRevision,
            _path: &str,
        ) -> Result<BlobRead, GitFailure> {
            self.0.set(self.0.get() + 1);
            Ok(BlobRead::Present("cited\n".into()))
        }
    }

    #[test]
    fn reads_each_blob_once_for_all_citations_in_a_path_group() {
        let make_site = |id: &str| EvidenceSite {
            owner: EvidenceOwner {
                kind: OwnerKind::Proposal,
                id: StableId::new("proposal").unwrap(),
                title: None,
                ratified: false,
            },
            ownership: RequirementOwnership::TargetRequirement {
                proposal_id: StableId::new("proposal").unwrap(),
                requirement_id: StableId::new("requirement").unwrap(),
            },
            source_id: StableId::new("source").unwrap(),
            repository: Utf8PathBuf::from("."),
            source_revision: Some("base".into()),
            revision: "base".into(),
            reference_id: StableId::new(id).unwrap(),
            path: "src/lib.rs".into(),
            line: Some(1),
        };
        let first = make_site("evidence_first");
        let second = make_site("evidence_second");
        let reader = CountingReader(Cell::new(0));
        let base = ResolvedRevision("base".into());
        let head = ResolvedRevision("head".into());
        let outcomes = group(
            &reader,
            Utf8Path::new("."),
            &base,
            &head,
            "src/lib.rs",
            "src/lib.rs",
            &[&first, &second],
        )
        .unwrap();
        assert_eq!(reader.0.get(), 2);
        assert_eq!(outcomes.len(), 2);
    }
}
