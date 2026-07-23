//! Publishes a complete wiki corpus as one fail-closed filesystem transaction.

use crate::wiki::model::WikiCorpus;
use camino::{Utf8Path, Utf8PathBuf};
use serde::Serialize;

mod error;
mod manifest;
mod stage;
mod transaction;

pub use error::PublishError;
use stage::generate_and_replace;
#[cfg(test)]
use stage::write_page;
#[cfg(test)]
use transaction::replace_output_with;
use transaction::{acquire_lock, preflight, TransactionPaths};

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
    publish_with(corpus, output, |path| std::fs::create_dir(path))
}

fn publish_with(
    corpus: &WikiCorpus,
    output: PublicationOutput,
    create_stage: impl FnOnce(&Utf8Path) -> std::io::Result<()>,
) -> Result<PublishReport, PublishError> {
    let paths = TransactionPaths::new(&output.path)?;
    let output_state = preflight(&output, &paths)?;
    let lock = acquire_lock(&paths)?;
    let (stage_created, result) = match create_stage(&paths.stage) {
        Ok(()) => (
            true,
            generate_and_replace(corpus, output, &paths, output_state),
        ),
        Err(error) => (
            false,
            Err(PublishError::io(
                "create staging directory",
                &paths.stage,
                error,
            )),
        ),
    };
    drop(lock);

    match result {
        Err(error @ PublishError::RollbackFailed { .. }) => Err(error),
        Err(error) => {
            if stage_created && paths.stage.exists() {
                // Never recursively remove through a mutable artifact pathname: a
                // replacement tree may have been installed there after staging.
                if let Err(cleanup) = std::fs::remove_dir(&paths.stage) {
                    return Err(PublishError::CleanupFailed {
                        primary: Box::new(error),
                        path: paths.stage,
                        cleanup,
                    });
                }
            }
            if let Err(cleanup) = std::fs::remove_file(&paths.lock) {
                return Err(PublishError::CleanupFailed {
                    primary: Box::new(error),
                    path: paths.lock,
                    cleanup,
                });
            }
            Err(error)
        }
        Ok(mut report) => {
            if let Err(error) = std::fs::remove_file(&paths.lock) {
                report.cleanup_warnings.push(CleanupWarning {
                    path: paths.lock,
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
