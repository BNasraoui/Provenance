use super::render::routes::{static_page_path, WIKI_CSS_ROUTE};
use super::render::RenderedPage;
use anyhow::{bail, Context};
use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

mod legacy;
mod transaction;

const MANIFEST_NAME: &str = ".provenance-wiki-output.json";
const GENERATOR: &str = "provenance-wiki";
const MANIFEST_VERSION: u8 = 1;

#[derive(Deserialize, Serialize)]
struct OwnershipManifest {
    generator: String,
    version: u8,
    files: Vec<String>,
}

pub(super) fn publish(
    out: &Utf8Path,
    pages: &[RenderedPage],
    stylesheet: &str,
) -> anyhow::Result<()> {
    let parent = output_parent(out)?;
    std::fs::create_dir_all(parent)
        .with_context(|| format!("failed to create wiki output parent {parent}"))?;
    let publication = transaction::Publication::begin(out)?;
    let stage = publication.create_stage()?;
    let result =
        stage_output(out, &stage, pages, stylesheet).and_then(|()| publication.commit(&stage));
    if let Err(error) = result {
        if let Err(cleanup_error) = publication.abort(&stage) {
            bail!("wiki publication failed: {error:#}; cleanup also failed: {cleanup_error:#}");
        }
        return Err(error);
    }
    Ok(())
}

fn stage_output(
    out: &Utf8Path,
    stage: &Utf8Path,
    pages: &[RenderedPage],
    stylesheet: &str,
) -> anyhow::Result<()> {
    if out.exists() {
        copy_tree(out.as_std_path(), stage.as_std_path())?;
    }
    let (owned, manifested) = match load_owned_files(out)? {
        Ownership::Manifest(files) => (files, true),
        Ownership::Missing => (legacy::owned_files(stage, pages)?, false),
        Ownership::UnrecognizedManifest => (Vec::new(), false),
    };
    for relative in owned {
        let path = stage.join(relative);
        remove_owned_file(&path)?;
        remove_empty_parents(stage, &path)?;
    }
    if manifested {
        remove_owned_file(&stage.join(MANIFEST_NAME))?;
    }

    let mut generated = BTreeSet::new();
    for page in pages {
        let path = static_page_path(stage, &page.route);
        write_claimed_file(stage, &path, page.html.as_bytes(), &mut generated)?;
    }
    let stylesheet_path = stage.join(WIKI_CSS_ROUTE.trim_start_matches('/'));
    write_claimed_file(
        stage,
        &stylesheet_path,
        stylesheet.as_bytes(),
        &mut generated,
    )?;
    write_manifest(stage, generated)
}

enum Ownership {
    Manifest(Vec<Utf8PathBuf>),
    Missing,
    UnrecognizedManifest,
}

fn load_owned_files(out: &Utf8Path) -> anyhow::Result<Ownership> {
    let path = out.join(MANIFEST_NAME);
    let source = match std::fs::read_to_string(&path) {
        Ok(source) => source,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Ownership::Missing)
        }
        Err(error) => return Err(error).with_context(|| format!("failed to read {path}")),
    };
    let manifest: OwnershipManifest = match serde_json::from_str(&source) {
        Ok(manifest) => manifest,
        Err(_) => return Ok(Ownership::UnrecognizedManifest),
    };
    if manifest.generator != GENERATOR || manifest.version != MANIFEST_VERSION {
        return Ok(Ownership::UnrecognizedManifest);
    }
    let mut owned = BTreeSet::new();
    for file in manifest.files {
        let relative = Utf8PathBuf::from(file);
        if !is_safe_relative(&relative) || relative == Utf8Path::new(MANIFEST_NAME) {
            bail!("wiki ownership manifest {path} contains unsafe path {relative}");
        }
        owned.insert(relative);
    }
    Ok(Ownership::Manifest(owned.into_iter().collect()))
}

fn is_safe_relative(path: &Utf8Path) -> bool {
    !path.as_str().is_empty()
        && path
            .components()
            .all(|component| matches!(component, camino::Utf8Component::Normal(_)))
}

fn write_claimed_file(
    stage: &Utf8Path,
    path: &Utf8Path,
    bytes: &[u8],
    generated: &mut BTreeSet<String>,
) -> anyhow::Result<()> {
    let relative = path
        .strip_prefix(stage)
        .with_context(|| format!("generated wiki path {path} escaped staging directory {stage}"))?;
    if path.exists() {
        bail!("refusing to replace unowned output path {relative}");
    }
    let parent = path
        .parent()
        .with_context(|| format!("generated wiki path {path} has no parent"))?;
    std::fs::create_dir_all(parent)
        .with_context(|| format!("failed to create staged wiki directory {parent}"))?;
    std::fs::write(path, bytes)
        .with_context(|| format!("failed to write staged wiki file {path}"))?;
    generated.insert(relative.as_str().to_string());
    Ok(())
}

fn write_manifest(stage: &Utf8Path, files: BTreeSet<String>) -> anyhow::Result<()> {
    let path = stage.join(MANIFEST_NAME);
    if path.exists() {
        bail!("refusing to replace unowned output path {MANIFEST_NAME}");
    }
    let manifest = OwnershipManifest {
        generator: GENERATOR.to_string(),
        version: MANIFEST_VERSION,
        files: files.into_iter().collect(),
    };
    let bytes = serde_json::to_vec_pretty(&manifest)?;
    std::fs::write(&path, bytes).with_context(|| format!("failed to write {path}"))
}

fn remove_owned_file(path: &Utf8Path) -> anyhow::Result<()> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => {
            Err(error).with_context(|| format!("failed to remove owned wiki file {path}"))
        }
    }
}

fn remove_empty_parents(stage: &Utf8Path, file: &Utf8Path) -> anyhow::Result<()> {
    let mut parent = file.parent();
    while let Some(directory) = parent.filter(|directory| *directory != stage) {
        match std::fs::remove_dir(directory) {
            Ok(()) => parent = directory.parent(),
            Err(error) if error.kind() == std::io::ErrorKind::DirectoryNotEmpty => break,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                parent = directory.parent();
            }
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("failed to prune legacy wiki directory {directory}"));
            }
        }
    }
    Ok(())
}

fn copy_tree(source: &std::path::Path, destination: &std::path::Path) -> anyhow::Result<()> {
    for entry in std::fs::read_dir(source)
        .with_context(|| format!("failed to read existing output {}", source.display()))?
    {
        let entry = entry?;
        let from = entry.path();
        let to = destination.join(entry.file_name());
        let metadata = std::fs::symlink_metadata(&from)?;
        if metadata.file_type().is_symlink() {
            bail!(
                "refusing to follow symlink in existing output {}",
                from.display()
            );
        }
        if metadata.is_dir() {
            std::fs::create_dir(&to)?;
            copy_tree(&from, &to)?;
        } else if metadata.is_file() {
            std::fs::hard_link(&from, &to).with_context(|| {
                format!(
                    "failed to preserve existing output file {} in staging",
                    from.display()
                )
            })?;
        } else {
            bail!("unsupported entry in existing output {}", from.display());
        }
    }
    Ok(())
}

fn output_parent(out: &Utf8Path) -> anyhow::Result<&Utf8Path> {
    out.parent()
        .filter(|parent| !parent.as_str().is_empty())
        .or_else(|| Some(Utf8Path::new(".")))
        .context("wiki output path has no parent")
}

fn output_name(out: &Utf8Path) -> anyhow::Result<&str> {
    out.file_name()
        .context("wiki output path must name a directory")
}
