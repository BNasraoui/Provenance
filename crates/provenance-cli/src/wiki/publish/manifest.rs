use super::{OutputPolicy, PublishError, GENERATOR, MANIFEST_VERSION, OWNERSHIP_MANIFEST};
use camino::Utf8Path;
use provenance_macros::rule;
use serde::{Deserialize, Serialize};
use std::io::Read;

const MAX_MANIFEST_BYTES: u64 = 4096;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct OwnershipManifest<'a> {
    #[serde(borrow)]
    pub generator: std::borrow::Cow<'a, str>,
    pub version: u32,
}

/// What the publisher found where its ownership marker would be, as data.
///
/// Every filesystem read and every parse happens before this value exists, so
/// the ownership decision below sees a finite set of states rather than a
/// directory.
#[derive(Debug, Clone, PartialEq, Eq)]
enum MarkerState {
    /// No marker: the directory has never been published by this generator.
    Absent,
    /// A marker this generator wrote and still understands.
    Owned,
    /// Well-formed marker naming a different generator.
    ForeignGenerator { generator: String },
    /// Our marker, written by a version whose layout we cannot assume.
    UnsupportedVersion { version: u32 },
    /// Too large to be one of our markers.
    Oversized,
    /// Present but unparseable.
    Malformed { detail: String },
    /// Something other than a regular file sits at the marker's name.
    NotRegularFile,
}

/// What the publisher may do with the directory it was pointed at.
#[derive(Debug, PartialEq, Eq)]
enum OwnershipVerdict {
    /// The whole directory may be replaced.
    Replace,
    /// Nothing proves the directory is ours and it is not empty.
    RefuseUnrecognizedDirectory,
    /// A marker is there but does not prove ownership.
    RefuseUnusableMarker { detail: String },
    /// Our marker, from a version we cannot act on.
    RefuseUnsupportedVersion { version: u32 },
}

/// Decides whether the wiki may replace the directory it was pointed at.
///
/// Replacing a directory destroys everything in it, so the publisher may only
/// do it when nothing of the caller's is at risk: the directory carries a
/// marker proving this generator published it before, or it is empty, or the
/// caller named a path the generator owns by construction (the default
/// output). A marker that exists but does not prove ownership is always a
/// refusal, never a licence to overwrite, whichever policy asked.
#[rule("rule_wiki_output_ownership")]
fn decide_output_ownership(
    policy: OutputPolicy,
    marker: &MarkerState,
    directory_is_empty: bool,
) -> OwnershipVerdict {
    match marker {
        MarkerState::Owned => OwnershipVerdict::Replace,
        MarkerState::Absent => {
            if policy == OutputPolicy::GeneratorOwned || directory_is_empty {
                OwnershipVerdict::Replace
            } else {
                OwnershipVerdict::RefuseUnrecognizedDirectory
            }
        }
        MarkerState::UnsupportedVersion { version } => {
            OwnershipVerdict::RefuseUnsupportedVersion { version: *version }
        }
        MarkerState::ForeignGenerator { generator } => OwnershipVerdict::RefuseUnusableMarker {
            detail: format!("unknown generator {generator:?}"),
        },
        MarkerState::Oversized => OwnershipVerdict::RefuseUnusableMarker {
            detail: format!("marker is too large (maximum {MAX_MANIFEST_BYTES} bytes)"),
        },
        MarkerState::Malformed { detail } => OwnershipVerdict::RefuseUnusableMarker {
            detail: detail.clone(),
        },
        MarkerState::NotRegularFile => OwnershipVerdict::RefuseUnusableMarker {
            detail: "marker is not a regular file".to_string(),
        },
    }
}

/// Reads the directory, then hands the decision to the rule above.
pub(super) fn validate_output_handle(
    directory: Option<std::fs::File>,
    output: &Utf8Path,
    policy: OutputPolicy,
) -> Result<(), PublishError> {
    let Some(directory) = directory else {
        return Ok(());
    };
    let manifest_path = output.join(OWNERSHIP_MANIFEST);
    let marker = read_marker_state(&directory, &manifest_path)?;
    let directory_is_empty = directory_is_empty_at(directory, output)?;
    match decide_output_ownership(policy, &marker, directory_is_empty) {
        OwnershipVerdict::Replace => Ok(()),
        OwnershipVerdict::RefuseUnrecognizedDirectory => {
            Err(PublishError::CustomOutputUnrecognized {
                path: output.to_path_buf(),
            })
        }
        OwnershipVerdict::RefuseUnusableMarker { detail } => Err(PublishError::InvalidManifest {
            path: manifest_path,
            detail,
        }),
        OwnershipVerdict::RefuseUnsupportedVersion { version } => {
            Err(PublishError::UnknownManifestVersion {
                path: manifest_path,
                version,
            })
        }
    }
}

fn read_marker_state(
    directory: &std::fs::File,
    manifest_path: &Utf8Path,
) -> Result<MarkerState, PublishError> {
    let mut options = fs_at::OpenOptions::default();
    options.read(true).follow(false);
    match options.open_at(directory, OWNERSHIP_MANIFEST) {
        Ok(file) => {
            if !file
                .metadata()
                .map_err(|error| {
                    PublishError::io("inspect ownership marker", manifest_path, error)
                })?
                .is_file()
            {
                return Ok(MarkerState::NotRegularFile);
            }
            let mut bytes = Vec::new();
            file.take(MAX_MANIFEST_BYTES + 1)
                .read_to_end(&mut bytes)
                .map_err(|error| PublishError::io("read", manifest_path, error))?;
            if bytes.len() as u64 > MAX_MANIFEST_BYTES {
                return Ok(MarkerState::Oversized);
            }
            Ok(classify_marker_bytes(&bytes))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(MarkerState::Absent),
        Err(error) => Err(PublishError::io(
            "inspect ownership marker",
            manifest_path,
            error,
        )),
    }
}

fn classify_marker_bytes(bytes: &[u8]) -> MarkerState {
    match serde_json::from_slice::<OwnershipManifest<'_>>(bytes) {
        Err(error) => MarkerState::Malformed {
            detail: error.to_string(),
        },
        Ok(manifest) if manifest.generator != GENERATOR => MarkerState::ForeignGenerator {
            generator: manifest.generator.into_owned(),
        },
        Ok(manifest) if manifest.version != MANIFEST_VERSION => MarkerState::UnsupportedVersion {
            version: manifest.version,
        },
        Ok(_) => MarkerState::Owned,
    }
}

fn directory_is_empty_at(
    mut directory: std::fs::File,
    path: &Utf8Path,
) -> Result<bool, PublishError> {
    let entries = fs_at::read_dir(&mut directory)
        .map_err(|error| PublishError::io("read custom output directory", path, error))?;
    for entry in entries {
        let entry =
            entry.map_err(|error| PublishError::io("read custom output directory", path, error))?;
        if entry.name() != "." && entry.name() != ".." {
            return Ok(false);
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::{decide_output_ownership, MarkerState, OutputPolicy, OwnershipVerdict};
    use provenance_macros::verifies;

    /// How a verdict disposes of the directory, without the wording carried
    /// along for the caller's error message.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Disposition {
        Replace,
        RefuseDirectory,
        RefuseMarker,
        RefuseVersion,
    }

    // The variant lists are built by an exhaustive match, so adding an
    // OutputPolicy or MarkerState variant fails compilation until the new
    // variant joins the chain, keeping the exhaustion proofs below complete.
    fn all_policies() -> Vec<OutputPolicy> {
        let mut all = vec![OutputPolicy::GeneratorOwned];
        while let Some(next) = match all.last().unwrap() {
            OutputPolicy::GeneratorOwned => Some(OutputPolicy::Custom),
            OutputPolicy::Custom => None,
        } {
            all.push(next);
        }
        all
    }

    fn all_marker_states() -> Vec<MarkerState> {
        let mut all = vec![MarkerState::Absent];
        while let Some(next) = match all.last().unwrap() {
            MarkerState::Absent => Some(MarkerState::Owned),
            MarkerState::Owned => Some(MarkerState::ForeignGenerator {
                generator: "some-other-generator".to_string(),
            }),
            MarkerState::ForeignGenerator { .. } => {
                Some(MarkerState::UnsupportedVersion { version: 99 })
            }
            MarkerState::UnsupportedVersion { .. } => Some(MarkerState::Oversized),
            MarkerState::Oversized => Some(MarkerState::Malformed {
                detail: "expected value at line 1 column 1".to_string(),
            }),
            MarkerState::Malformed { .. } => Some(MarkerState::NotRegularFile),
            MarkerState::NotRegularFile => None,
        } {
            all.push(next);
        }
        all
    }

    /// Every point of the decision's domain: policy x marker state x
    /// emptiness.
    fn whole_domain() -> Vec<(OutputPolicy, MarkerState, bool)> {
        let mut domain = Vec::new();
        for policy in all_policies() {
            for marker in all_marker_states() {
                for directory_is_empty in [false, true] {
                    domain.push((policy, marker.clone(), directory_is_empty));
                }
            }
        }
        domain
    }

    fn disposition(verdict: &OwnershipVerdict) -> Disposition {
        match verdict {
            OwnershipVerdict::Replace => Disposition::Replace,
            OwnershipVerdict::RefuseUnrecognizedDirectory => Disposition::RefuseDirectory,
            OwnershipVerdict::RefuseUnusableMarker { .. } => Disposition::RefuseMarker,
            OwnershipVerdict::RefuseUnsupportedVersion { .. } => Disposition::RefuseVersion,
        }
    }

    // Independent restatement of the decision the rule is meant to make,
    // written from the rule's statement rather than from its branches: the
    // wiki may replace a directory it proved it published, an empty
    // directory, or the output it owns by construction; a marker it cannot
    // act on refuses on its own terms. Must not be implemented by calling
    // decide_output_ownership.
    fn expected_disposition(
        policy: OutputPolicy,
        marker: &MarkerState,
        directory_is_empty: bool,
    ) -> Disposition {
        match marker {
            MarkerState::Owned => Disposition::Replace,
            MarkerState::Absent => {
                if directory_is_empty || policy == OutputPolicy::GeneratorOwned {
                    Disposition::Replace
                } else {
                    Disposition::RefuseDirectory
                }
            }
            MarkerState::UnsupportedVersion { .. } => Disposition::RefuseVersion,
            MarkerState::ForeignGenerator { .. }
            | MarkerState::Oversized
            | MarkerState::Malformed { .. }
            | MarkerState::NotRegularFile => Disposition::RefuseMarker,
        }
    }

    // Independent restatement of the safety property the rule exists for
    // ("replacing must destroy nothing the caller owns"), used as the oracle
    // below. Weaker than the decision itself: it says what may never be
    // replaced, not what must be.
    fn replacing_can_destroy_callers_files(
        policy: OutputPolicy,
        marker: &MarkerState,
        directory_is_empty: bool,
    ) -> bool {
        let published_by_us = *marker == MarkerState::Owned;
        let generator_owns_the_path = policy == OutputPolicy::GeneratorOwned;
        !directory_is_empty && !published_by_us && !generator_owns_the_path
    }

    #[test]
    #[verifies("rule_wiki_output_ownership", exhaustion)]
    fn every_decision_in_the_domain_matches_the_rule() {
        let domain = whole_domain();
        assert_eq!(domain.len(), 28, "the decision's domain changed size");
        for (policy, marker, directory_is_empty) in domain {
            let verdict = decide_output_ownership(policy, &marker, directory_is_empty);
            assert_eq!(
                disposition(&verdict),
                expected_disposition(policy, &marker, directory_is_empty),
                "wrong verdict {verdict:?} for {policy:?} marker {marker:?} \
                 (directory_is_empty={directory_is_empty})"
            );
        }
    }

    #[test]
    #[verifies("rule_wiki_output_ownership", exhaustion)]
    fn never_replaces_a_directory_holding_files_it_cannot_claim() {
        for (policy, marker, directory_is_empty) in whole_domain() {
            if replacing_can_destroy_callers_files(policy, &marker, directory_is_empty) {
                let verdict = decide_output_ownership(policy, &marker, directory_is_empty);
                assert_ne!(
                    verdict,
                    OwnershipVerdict::Replace,
                    "replacing a directory of the caller's files was allowed for {policy:?} \
                     marker {marker:?}"
                );
            }
        }
    }

    #[test]
    #[verifies("rule_wiki_output_ownership", exhaustion)]
    fn a_marker_it_cannot_act_on_never_permits_replacement() {
        for (policy, marker, directory_is_empty) in whole_domain() {
            let unusable = !matches!(marker, MarkerState::Absent | MarkerState::Owned);
            if unusable {
                assert_ne!(
                    decide_output_ownership(policy, &marker, directory_is_empty),
                    OwnershipVerdict::Replace,
                    "an unusable marker {marker:?} was overridden by {policy:?} \
                     (directory_is_empty={directory_is_empty})"
                );
            }
        }
    }
}
