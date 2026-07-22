mod git;
mod projection;

use camino::Utf8Path;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub use projection::GraphExport;

use git::{GitRepository, TreeSource};
use projection::load_projection;

const STORE_PATH: &str = ".provenance/state";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalCorrelation {
    pub system: String,
    pub key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphReference {
    pub schema_version: u32,
    pub reference_id: String,
    pub repository_id: String,
    pub store_path: String,
    pub scope_id: String,
    pub commit: String,
    pub graph_digest: String,
    #[serde(
        default,
        deserialize_with = "deserialize_correlation",
        skip_serializing_if = "Option::is_none"
    )]
    pub correlation: Option<ExternalCorrelation>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum GraphReferenceError {
    #[error("missing graph reference data: {detail}")]
    Missing { detail: String },
    #[error("mismatched graph reference {field}: expected {expected}, actual {actual}")]
    Mismatched {
        field: &'static str,
        expected: String,
        actual: String,
    },
    #[error("incomplete graph reference: {detail}")]
    Incomplete { detail: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GraphReferenceSummary {
    pub schema_version: u32,
    pub operation: &'static str,
    pub reference: GraphReference,
    pub counts: GraphCounts,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Verification {
    pub schema_version: u32,
    pub operation: &'static str,
    pub valid: bool,
    pub reference_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ExactExport {
    pub schema_version: u32,
    pub operation: &'static str,
    pub reference_id: String,
    pub graph: GraphExport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GraphCounts {
    pub sources: usize,
    pub domains: usize,
    pub requirements: usize,
    pub boundaries: usize,
    pub topics: usize,
    pub questions: usize,
    pub resolutions: usize,
    pub rules: usize,
    pub services: usize,
    pub service_bindings: usize,
    pub edges: usize,
}

pub struct GraphReferences {
    repository: GitRepository,
}

impl GraphReference {
    pub fn from_json(bytes: &[u8]) -> Result<Self, GraphReferenceError> {
        let reference: Self =
            serde_json::from_slice(bytes).map_err(|error| GraphReferenceError::Incomplete {
                detail: format!("reference JSON is invalid: {error}"),
            })?;
        if reference.schema_version != 1 {
            return Err(GraphReferenceError::Incomplete {
                detail: format!(
                    "unsupported schema_version {}; expected 1",
                    reference.schema_version
                ),
            });
        }
        validate_prefixed_hash("reference_id", &reference.reference_id, "grf1_", 64)?;
        validate_prefixed_hash("repository_id", &reference.repository_id, "git1_", 64)?;
        if reference.store_path != STORE_PATH {
            return Err(GraphReferenceError::Incomplete {
                detail: format!("store_path must be '{STORE_PATH}'"),
            });
        }
        provenance_core::ScopeId::new(reference.scope_id.clone()).map_err(incomplete)?;
        if !matches!(reference.commit.len(), 40 | 64)
            || !reference.commit.bytes().all(is_lower_hex_digit)
        {
            return Err(GraphReferenceError::Incomplete {
                detail: "commit must be a full 40- or 64-character hexadecimal object ID".into(),
            });
        }
        validate_prefixed_hash("graph_digest", &reference.graph_digest, "sha256:", 64)?;
        if let Some(correlation) = &reference.correlation {
            validate_correlation(correlation)?;
        }
        Ok(reference)
    }
}

impl GraphReferences {
    pub fn open(repo: &Utf8Path) -> Result<Self, GraphReferenceError> {
        Ok(Self {
            repository: GitRepository::open(repo)?,
        })
    }

    pub fn issue(
        &self,
        scope: &str,
        revision: Option<&str>,
        correlation: Option<ExternalCorrelation>,
    ) -> Result<GraphReference, GraphReferenceError> {
        if let Some(correlation) = &correlation {
            validate_correlation(correlation)?;
        }
        let implicit_head = revision.is_none();
        let commit = self.repository.resolve_commit(revision.unwrap_or("HEAD"))?;
        let graph = self.projection(TreeSource::Commit(&commit), scope)?;
        if implicit_head {
            let index = self.projection(TreeSource::Index, scope)?;
            let worktree = load_projection(self.repository.root(), scope)?;
            let committed_bytes = canonical_bytes(&graph)?;
            if committed_bytes != canonical_bytes(&index)?
                || committed_bytes != canonical_bytes(&worktree)?
            {
                return Err(GraphReferenceError::Incomplete {
                    detail: format!(
                        "implicit HEAD requires clean canonical state for scope '{scope}'; commit graph changes first"
                    ),
                });
            }
        }

        let repository_id = self.repository.identity(&commit)?;
        let graph_digest = digest(&canonical_bytes(&graph)?);
        let reference_id =
            reference_identity(&repository_id, STORE_PATH, scope, &commit, &graph_digest);
        Ok(GraphReference {
            schema_version: 1,
            reference_id,
            repository_id,
            store_path: STORE_PATH.to_string(),
            scope_id: scope.to_string(),
            commit,
            graph_digest,
            correlation,
        })
    }

    pub fn show(
        &self,
        reference: &GraphReference,
    ) -> Result<GraphReferenceSummary, GraphReferenceError> {
        let graph = self.verify_and_load(reference)?;
        Ok(GraphReferenceSummary {
            schema_version: 1,
            operation: "show",
            reference: reference.clone(),
            counts: GraphCounts::from(&graph),
        })
    }

    pub fn verify(&self, reference: &GraphReference) -> Result<Verification, GraphReferenceError> {
        self.verify_and_load(reference)?;
        Ok(Verification {
            schema_version: 1,
            operation: "verify",
            valid: true,
            reference_id: reference.reference_id.clone(),
        })
    }

    pub fn exact_export(
        &self,
        reference: &GraphReference,
    ) -> Result<ExactExport, GraphReferenceError> {
        Ok(ExactExport {
            schema_version: 1,
            operation: "exact-export",
            reference_id: reference.reference_id.clone(),
            graph: self.verify_and_load(reference)?,
        })
    }

    fn projection(
        &self,
        source: TreeSource<'_>,
        scope: &str,
    ) -> Result<GraphExport, GraphReferenceError> {
        let tree = self.repository.materialize(source)?;
        load_projection(
            Utf8Path::from_path(tree.path()).ok_or_else(|| GraphReferenceError::Incomplete {
                detail: "temporary Git tree path is not UTF-8".into(),
            })?,
            scope,
        )
    }

    fn verify_and_load(
        &self,
        reference: &GraphReference,
    ) -> Result<GraphExport, GraphReferenceError> {
        if reference.store_path != STORE_PATH {
            return mismatch("store_path", STORE_PATH, &reference.store_path);
        }
        let commit = self.repository.resolve_commit(&reference.commit)?;
        if commit != reference.commit {
            return mismatch("commit", &reference.commit, &commit);
        }
        let repository_id = self.repository.identity(&commit)?;
        if repository_id != reference.repository_id {
            return mismatch("repository_id", &reference.repository_id, &repository_id);
        }
        let graph = self.projection(TreeSource::Commit(&commit), &reference.scope_id)?;
        let graph_digest = digest(&canonical_bytes(&graph)?);
        if graph_digest != reference.graph_digest {
            return mismatch("graph_digest", &reference.graph_digest, &graph_digest);
        }
        let identity = reference_identity(
            &repository_id,
            STORE_PATH,
            &reference.scope_id,
            &commit,
            &graph_digest,
        );
        if identity != reference.reference_id {
            return mismatch("reference_id", &reference.reference_id, &identity);
        }
        Ok(graph)
    }
}

impl ExactExport {
    pub fn from_json(bytes: &[u8]) -> Result<Self, GraphReferenceError> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Document {
            schema_version: u32,
            operation: String,
            reference_id: String,
            graph: GraphExport,
        }

        let mut unknown = None;
        let mut deserializer = serde_json::Deserializer::from_slice(bytes);
        let document: Document = serde_ignored::deserialize(&mut deserializer, |path| {
            if unknown.is_none() {
                unknown = Some(path.to_string());
            }
        })
        .map_err(incomplete)?;
        if let Some(path) = unknown {
            return Err(incomplete(format!("unknown field `{path}`")));
        }
        if document.schema_version != 1 || document.operation != "exact-export" {
            return Err(GraphReferenceError::Incomplete {
                detail: "exact export must use schema_version 1 and operation 'exact-export'"
                    .into(),
            });
        }
        validate_prefixed_hash("reference_id", &document.reference_id, "grf1_", 64)?;
        if document.graph.schema_version != 1 {
            return Err(GraphReferenceError::Incomplete {
                detail: "graph schema_version must be 1".into(),
            });
        }
        document.graph.validate_schema_versions()?;
        document.graph.validate_no_collaboration_fields()?;
        projection::validate_scope_ownership(&document.graph, &document.graph.scope.id)?;
        Ok(Self {
            schema_version: 1,
            operation: "exact-export",
            reference_id: document.reference_id,
            graph: document.graph,
        })
    }
}

impl From<&GraphExport> for GraphCounts {
    fn from(graph: &GraphExport) -> Self {
        Self {
            sources: graph.sources.len(),
            domains: graph.domains.len(),
            requirements: graph.requirements.len(),
            boundaries: graph.boundaries.len(),
            topics: graph.topics.len(),
            questions: graph.questions.len(),
            resolutions: graph.resolutions.len(),
            rules: graph.rules.len(),
            services: graph.services.len(),
            service_bindings: graph.service_bindings.len(),
            edges: graph.edges.len(),
        }
    }
}

fn mismatch<T>(
    field: &'static str,
    expected: &str,
    actual: &str,
) -> Result<T, GraphReferenceError> {
    Err(GraphReferenceError::Mismatched {
        field,
        expected: expected.to_string(),
        actual: actual.to_string(),
    })
}

fn validate_correlation(correlation: &ExternalCorrelation) -> Result<(), GraphReferenceError> {
    if correlation.system.trim().is_empty() || correlation.key.trim().is_empty() {
        return Err(GraphReferenceError::Incomplete {
            detail: "external correlation system and key must not be empty".into(),
        });
    }
    Ok(())
}

fn deserialize_correlation<'de, D>(deserializer: D) -> Result<Option<ExternalCorrelation>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    ExternalCorrelation::deserialize(deserializer).map(Some)
}

fn validate_prefixed_hash(
    field: &str,
    value: &str,
    prefix: &str,
    digits: usize,
) -> Result<(), GraphReferenceError> {
    let Some(hash) = value.strip_prefix(prefix) else {
        return Err(GraphReferenceError::Incomplete {
            detail: format!("{field} must start with '{prefix}'"),
        });
    };
    if hash.len() != digits || !hash.bytes().all(is_lower_hex_digit) {
        return Err(GraphReferenceError::Incomplete {
            detail: format!("{field} must contain {digits} hexadecimal characters"),
        });
    }
    Ok(())
}

const fn is_lower_hex_digit(byte: u8) -> bool {
    byte.is_ascii_digit() || matches!(byte, b'a'..=b'f')
}

fn reference_identity(
    repository_id: &str,
    store_path: &str,
    scope: &str,
    commit: &str,
    graph_digest: &str,
) -> String {
    let framed = format!(
        "graph-reference-v1\0{repository_id}\0{store_path}\0{scope}\0{commit}\0{graph_digest}"
    );
    format!("grf1_{}", sha256(framed.as_bytes()))
}

fn canonical_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, GraphReferenceError> {
    let value = serde_json::to_value(value).map_err(incomplete)?;
    let mut bytes = Vec::new();
    write_canonical_json(&value, &mut bytes)?;
    Ok(bytes)
}

fn write_canonical_json(
    value: &serde_json::Value,
    output: &mut Vec<u8>,
) -> Result<(), GraphReferenceError> {
    match value {
        serde_json::Value::Object(map) => {
            output.push(b'{');
            let mut entries: Vec<_> = map.iter().collect();
            entries.sort_by_key(|(key, _)| key.as_str());
            for (index, (key, value)) in entries.into_iter().enumerate() {
                if index > 0 {
                    output.push(b',');
                }
                output.extend(serde_json::to_vec(key).map_err(incomplete)?);
                output.push(b':');
                write_canonical_json(value, output)?;
            }
            output.push(b'}');
        }
        serde_json::Value::Array(values) => {
            output.push(b'[');
            for (index, value) in values.iter().enumerate() {
                if index > 0 {
                    output.push(b',');
                }
                write_canonical_json(value, output)?;
            }
            output.push(b']');
        }
        _ => output.extend(serde_json::to_vec(value).map_err(incomplete)?),
    }
    Ok(())
}

fn digest(bytes: &[u8]) -> String {
    format!("sha256:{}", sha256(bytes))
}

fn sha256(bytes: &[u8]) -> String {
    use std::fmt::Write;

    let digest = Sha256::digest(bytes);
    digest.iter().fold(
        String::with_capacity(digest.len() * 2),
        |mut output, byte| {
            write!(output, "{byte:02x}").expect("writing to a String cannot fail");
            output
        },
    )
}

fn incomplete(error: impl std::fmt::Display) -> GraphReferenceError {
    GraphReferenceError::Incomplete {
        detail: error.to_string(),
    }
}
