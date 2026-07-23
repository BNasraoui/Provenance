use super::manifest::OwnershipManifest;
use super::transaction::{replace_output, OutputState, StageIdentity, TransactionDirectory};
use super::{
    PublicationOutput, PublishError, PublishReport, PublishedPage, GENERATOR, MANIFEST_VERSION,
    OWNERSHIP_MANIFEST,
};
use crate::wiki::model::WikiCorpus;
use crate::wiki::{render, theme};
use camino::Utf8Path;
use std::fs::File;
use std::io::Write;

pub(super) struct StageDirectory {
    root: File,
    identity: StageIdentity,
}

impl StageDirectory {
    pub(super) fn from_file(root: File, path: &Utf8Path) -> Result<Self, PublishError> {
        let identity = StageIdentity::from_file(&root)
            .map_err(|error| PublishError::io("record staging directory identity", path, error))?;
        Ok(Self { root, identity })
    }

    pub(super) const fn identity(&self) -> &StageIdentity {
        &self.identity
    }

    fn write_file(
        &self,
        relative: &[&str],
        contents: &[u8],
        display_path: &Utf8Path,
    ) -> Result<(), PublishError> {
        let mut directory = self.root.try_clone().map_err(|error| {
            PublishError::io("clone staging directory handle", display_path, error)
        })?;
        for segment in &relative[..relative.len() - 1] {
            directory = match fs_at::OpenOptions::default().mkdir_at(&directory, *segment) {
                Ok(created) => created,
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    let mut options = fs_at::OpenOptions::default();
                    options.read(true).follow(false);
                    options.open_dir_at(&directory, *segment).map_err(|error| {
                        PublishError::io("open staged directory", display_path, error)
                    })?
                }
                Err(error) => {
                    return Err(PublishError::io(
                        "create staged directory",
                        display_path,
                        error,
                    ))
                }
            };
        }
        let mut options = fs_at::OpenOptions::default();
        options
            .write(fs_at::OpenOptionsWriteMode::Write)
            .create_new(true)
            .follow(false);
        let mut file = options
            .open_at(&directory, relative[relative.len() - 1])
            .map_err(|error| PublishError::io("create staged file", display_path, error))?;
        file.write_all(contents)
            .map_err(|error| PublishError::io("write staged file", display_path, error))
    }
}

pub(super) fn generate_and_replace(
    corpus: &WikiCorpus,
    output: PublicationOutput,
    transaction: &TransactionDirectory,
    output_state: OutputState,
    stage: &StageDirectory,
) -> Result<PublishReport, PublishError> {
    let paths = &transaction.paths;
    let pages = render::render_corpus(corpus);
    for page in &pages {
        write_page_in(stage, &paths.stage, &page.route, &page.html)?;
    }
    let stylesheet = paths.stage.join("assets/provenance-wiki.css");
    stage.write_file(
        &["assets", "provenance-wiki.css"],
        theme::WIKI_CSS.as_bytes(),
        &stylesheet,
    )?;
    let manifest = serde_json::to_vec_pretty(&OwnershipManifest {
        generator: GENERATOR.into(),
        version: MANIFEST_VERSION,
    })
    .expect("ownership manifest is always serializable");
    let manifest_path = paths.stage.join(OWNERSHIP_MANIFEST);
    stage.write_file(&[OWNERSHIP_MANIFEST], &manifest, &manifest_path)?;
    transaction.validate_output(
        output.path.file_name().expect("validated output leaf"),
        &output.path,
        output.policy,
    )?;
    let cleanup_warnings = replace_output(
        &output.path,
        output.policy,
        transaction,
        output_state,
        &stage.identity,
    )?;

    Ok(PublishReport {
        status: "ok",
        scope: corpus.scope.clone(),
        out: output.path,
        page_count: pages.len(),
        pages: pages
            .into_iter()
            .map(|page| PublishedPage {
                route: page.route,
                title: page.title,
            })
            .collect(),
        cleanup_warnings,
    })
}

#[cfg(test)]
pub(super) fn write_page(stage: &Utf8Path, route: &str, html: &str) -> Result<(), PublishError> {
    let root = File::open(stage)
        .map_err(|error| PublishError::io("open staging directory", stage, error))?;
    let stage_directory = StageDirectory::from_file(root, stage)?;
    write_page_in(&stage_directory, stage, route, html)
}

fn write_page_in(
    stage_directory: &StageDirectory,
    stage: &Utf8Path,
    route: &str,
    html: &str,
) -> Result<(), PublishError> {
    if !route.starts_with('/') || !route.ends_with('/') {
        return Err(PublishError::InvalidRoute {
            route: route.to_string(),
            detail: "routes must begin and end with '/'".to_string(),
        });
    }
    let mut relative = Vec::new();
    let mut path = stage.to_path_buf();
    for segment in route.split('/').filter(|segment| !segment.is_empty()) {
        if matches!(segment, "." | "..") || segment.contains(['\\', '\0']) {
            return Err(PublishError::InvalidRoute {
                route: route.to_string(),
                detail: format!("unsafe path segment {segment:?}"),
            });
        }
        relative.push(segment);
        path.push(segment);
    }
    relative.push("index.html");
    path.push("index.html");
    stage_directory.write_file(&relative, html.as_bytes(), &path)
}
