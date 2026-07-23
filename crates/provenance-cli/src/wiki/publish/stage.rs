use super::manifest::OwnershipManifest;
use super::transaction::{replace_output, OutputState, TransactionPaths};
use super::{
    PublicationOutput, PublishError, PublishReport, PublishedPage, GENERATOR, MANIFEST_VERSION,
    OWNERSHIP_MANIFEST,
};
use crate::wiki::model::WikiCorpus;
use crate::wiki::{render, theme};
use camino::Utf8Path;

pub(super) fn generate_and_replace(
    corpus: &WikiCorpus,
    output: PublicationOutput,
    paths: &TransactionPaths,
    output_state: OutputState,
) -> Result<PublishReport, PublishError> {
    let pages = render::render_corpus(corpus);
    for page in &pages {
        write_page(&paths.stage, &page.route, &page.html)?;
    }
    let stylesheet = paths.stage.join("assets/provenance-wiki.css");
    std::fs::create_dir_all(stylesheet.parent().expect("stylesheet path has a parent")).map_err(
        |error| {
            PublishError::io(
                "create stylesheet directory",
                stylesheet.parent().expect("stylesheet path has a parent"),
                error,
            )
        },
    )?;
    std::fs::write(&stylesheet, theme::WIKI_CSS)
        .map_err(|error| PublishError::io("write stylesheet", &stylesheet, error))?;
    let manifest = serde_json::to_vec_pretty(&OwnershipManifest {
        generator: GENERATOR.into(),
        version: MANIFEST_VERSION,
    })
    .expect("ownership manifest is always serializable");
    let manifest_path = paths.stage.join(OWNERSHIP_MANIFEST);
    std::fs::write(&manifest_path, manifest)
        .map_err(|error| PublishError::io("write ownership marker", &manifest_path, error))?;
    super::manifest::validate_output(&output.path, output.policy)?;
    let cleanup_warnings = replace_output(&output.path, output.policy, paths, output_state)?;

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

pub(super) fn write_page(stage: &Utf8Path, route: &str, html: &str) -> Result<(), PublishError> {
    if !route.starts_with('/') || !route.ends_with('/') {
        return Err(PublishError::InvalidRoute {
            route: route.to_string(),
            detail: "routes must begin and end with '/'".to_string(),
        });
    }
    let mut path = stage.to_path_buf();
    for segment in route.split('/').filter(|segment| !segment.is_empty()) {
        if matches!(segment, "." | "..") || segment.contains(['\\', '\0']) {
            return Err(PublishError::InvalidRoute {
                route: route.to_string(),
                detail: format!("unsafe path segment {segment:?}"),
            });
        }
        path.push(segment);
    }
    path.push("index.html");
    std::fs::create_dir_all(path.parent().expect("page path has a parent")).map_err(|error| {
        PublishError::io(
            "create page directory",
            path.parent().expect("page path has a parent"),
            error,
        )
    })?;
    std::fs::write(&path, html)
        .map_err(|error| PublishError::io("write staged page", &path, error))?;
    Ok(())
}
