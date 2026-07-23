//! Publishes a complete wiki corpus as one fail-closed filesystem transaction.

use crate::wiki::model::WikiCorpus;
use camino::{Utf8Path, Utf8PathBuf};
use serde::Serialize;

mod error;
mod manifest;
mod stage;
mod transaction;

pub use error::PublishError;
#[cfg(test)]
use stage::write_page;
use stage::{generate_and_replace, StageDirectory};
#[cfg(test)]
use transaction::replace_output_with;
#[cfg(test)]
use transaction::TransactionPaths;
use transaction::{acquire_lock, preflight, TransactionDirectory};

pub const OWNERSHIP_MANIFEST: &str = ".provenance-wiki-output.json";

const GENERATOR: &str = "provenance-wiki";
const MANIFEST_VERSION: u32 = 1;

#[derive(Debug, Clone)]
pub struct PublicationOutput {
    path: Utf8PathBuf,
    policy: OutputPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputPolicy {
    GeneratorOwned,
    Custom,
}

impl PublicationOutput {
    pub const fn generator_owned(path: Utf8PathBuf) -> Self {
        Self {
            path,
            policy: OutputPolicy::GeneratorOwned,
        }
    }

    pub const fn custom(path: Utf8PathBuf) -> Self {
        Self {
            path,
            policy: OutputPolicy::Custom,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PublishReport {
    pub status: &'static str,
    pub scope: String,
    pub out: Utf8PathBuf,
    pub page_count: usize,
    pub pages: Vec<PublishedPage>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cleanup_warnings: Vec<CleanupWarning>,
}

#[derive(Debug, Serialize)]
pub struct PublishedPage {
    pub route: String,
    pub title: String,
}

#[derive(Debug, Serialize)]
pub struct CleanupWarning {
    pub path: Utf8PathBuf,
    pub action: &'static str,
    pub error: String,
}

pub fn publish(
    corpus: &WikiCorpus,
    output: PublicationOutput,
) -> Result<PublishReport, PublishError> {
    publish_with(corpus, output, |_| Ok(()))
}

fn publish_with(
    corpus: &WikiCorpus,
    output: PublicationOutput,
    after_stage_opened: impl FnOnce(&Utf8Path) -> std::io::Result<()>,
) -> Result<PublishReport, PublishError> {
    let transaction = TransactionDirectory::open(&output.path)?;
    let output_state = preflight(&output, &transaction)?;
    let lock = acquire_lock(&transaction)?;
    let paths = &transaction.paths;
    let (stage_created, result) = match transaction.create_stage() {
        Ok(stage_root) => {
            let result = StageDirectory::from_file(stage_root, &paths.stage).and_then(|stage| {
                after_stage_opened(&paths.stage).map_err(|error| {
                    PublishError::io("prepare staging directory", &paths.stage, error)
                })?;
                generate_and_replace(corpus, output, &transaction, output_state, &stage)
            });
            (true, result)
        }
        Err(error) => (
            false,
            Err(PublishError::io(
                "create staging directory",
                &paths.stage,
                error,
            )),
        ),
    };
    match result {
        Err(error @ PublishError::RollbackFailed { .. }) => Err(error),
        Err(error) => {
            if stage_created {
                // Never recursively remove through a mutable artifact pathname: a
                // replacement tree may have been installed there after staging.
                if let Err(cleanup) = transaction.remove_stage() {
                    let error = PublishError::CleanupFailed {
                        primary: Box::new(error),
                        path: paths.stage.clone(),
                        cleanup,
                    };
                    return match lock.cleanup(&transaction) {
                        Ok(()) => Err(error),
                        Err(cleanup) => Err(PublishError::CleanupFailed {
                            primary: Box::new(error),
                            path: paths.lock.clone(),
                            cleanup,
                        }),
                    };
                }
            }
            if let Err(cleanup) = lock.cleanup(&transaction) {
                return Err(PublishError::CleanupFailed {
                    primary: Box::new(error),
                    path: paths.lock.clone(),
                    cleanup,
                });
            }
            Err(error)
        }
        Ok(mut report) => {
            if let Err(error) = lock.cleanup(&transaction) {
                report.cleanup_warnings.push(CleanupWarning {
                    path: paths.lock.clone(),
                    action: "remove committed publication lock",
                    error: error.to_string(),
                });
            }
            if !report.cleanup_warnings.is_empty() {
                report.status = "ok_with_cleanup_required";
            }
            Ok(report)
        }
    }
}

#[cfg(test)]
mod tests;
