use super::manifest::OwnershipManifest;
use super::transaction::{replace_output, OutputState, StageIdentity, TransactionPaths};
use super::{
    PublicationOutput, PublishError, PublishReport, PublishedPage, GENERATOR, MANIFEST_VERSION,
    OWNERSHIP_MANIFEST,
};
use crate::wiki::model::WikiCorpus;
use crate::wiki::{render, theme};
use camino::Utf8Path;
use std::fs::File;
use std::io::Write;

#[cfg(unix)]
use rustix::fs::{mkdirat, openat, Mode, OFlags};

pub(super) struct StageDirectory {
    root: File,
    identity: StageIdentity,
}

impl StageDirectory {
    pub(super) fn open(path: &Utf8Path) -> Result<Self, PublishError> {
        #[cfg(unix)]
        let root = File::from(
            openat(
                rustix::fs::CWD,
                path.as_std_path(),
                OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                Mode::empty(),
            )
            .map_err(|error| PublishError::io("open staging directory", path, error.into()))?,
        );
        #[cfg(not(unix))]
        let root = File::open(path)
            .map_err(|error| PublishError::io("open staging directory", path, error))?;
        let identity = StageIdentity::from_file(&root)
            .map_err(|error| PublishError::io("record staging directory identity", path, error))?;
        Ok(Self { root, identity })
    }

    #[cfg(unix)]
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
            match mkdirat(&directory, *segment, Mode::from_raw_mode(0o755)) {
                Ok(()) => {}
                Err(error) if error == rustix::io::Errno::EXIST => {}
                Err(error) => {
                    return Err(PublishError::io(
                        "create staged directory",
                        display_path,
                        error.into(),
                    ));
                }
            }
            directory = File::from(
                openat(
                    &directory,
                    *segment,
                    OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                    Mode::empty(),
                )
                .map_err(|error| {
                    PublishError::io("open staged directory", display_path, error.into())
                })?,
            );
        }
        let mut file = File::from(
            openat(
                &directory,
                relative[relative.len() - 1],
                OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                Mode::from_raw_mode(0o644),
            )
            .map_err(|error| PublishError::io("create staged file", display_path, error.into()))?,
        );
        file.write_all(contents)
            .map_err(|error| PublishError::io("write staged file", display_path, error))
    }

    #[cfg(not(unix))]
    fn write_file(
        &self,
        relative: &[&str],
        contents: &[u8],
        display_path: &Utf8Path,
    ) -> Result<(), PublishError> {
        let root = display_path
            .ancestors()
            .nth(relative.len())
            .expect("display path contains every relative component");
        let path = relative
            .iter()
            .fold(root.to_path_buf(), |path, segment| path.join(segment));
        std::fs::create_dir_all(path.parent().expect("staged file has a parent"))
            .map_err(|error| PublishError::io("create staged directory", &path, error))?;
        std::fs::write(&path, contents)
            .map_err(|error| PublishError::io("write staged file", &path, error))
    }
}

pub(super) fn generate_and_replace(
    corpus: &WikiCorpus,
    output: PublicationOutput,
    paths: &TransactionPaths,
    output_state: OutputState,
    stage: &StageDirectory,
) -> Result<PublishReport, PublishError> {
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
    super::manifest::validate_output(&output.path, output.policy)?;
    let cleanup_warnings = replace_output(
        &output.path,
        output.policy,
        paths,
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
    let stage_directory = StageDirectory::open(stage)?;
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
