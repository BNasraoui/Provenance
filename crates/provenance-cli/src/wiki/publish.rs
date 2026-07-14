use super::render::routes::{static_page_path, WIKI_CSS_ROUTE};
use super::render::RenderedPage;
use anyhow::{bail, Context};
use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::sync::atomic::{AtomicU64, Ordering};

const MANIFEST_NAME: &str = ".provenance-wiki-output.json";
const GENERATOR: &str = "provenance-wiki";
const MANIFEST_VERSION: u8 = 1;
static UNIQUE_ID: AtomicU64 = AtomicU64::new(0);

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
    let stage = create_unique_dir(parent, output_name(out)?, "stage")?;
    let result =
        stage_output(out, &stage, pages, stylesheet).and_then(|()| replace_output(out, &stage));
    if result.is_err() && stage.exists() {
        let _ = std::fs::remove_dir_all(&stage);
    }
    result
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
    if let Some(owned) = load_owned_files(out)? {
        for relative in owned {
            remove_owned_file(&stage.join(relative))?;
        }
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

fn load_owned_files(out: &Utf8Path) -> anyhow::Result<Option<Vec<Utf8PathBuf>>> {
    let path = out.join(MANIFEST_NAME);
    let source = match std::fs::read_to_string(&path) {
        Ok(source) => source,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error).with_context(|| format!("failed to read {path}")),
    };
    let manifest: OwnershipManifest = match serde_json::from_str(&source) {
        Ok(manifest) => manifest,
        Err(_) => return Ok(None),
    };
    if manifest.generator != GENERATOR || manifest.version != MANIFEST_VERSION {
        return Ok(None);
    }
    let mut owned = BTreeSet::new();
    for file in manifest.files {
        let relative = Utf8PathBuf::from(file);
        if !is_safe_relative(&relative) || relative == Utf8Path::new(MANIFEST_NAME) {
            bail!("wiki ownership manifest {path} contains unsafe path {relative}");
        }
        owned.insert(relative);
    }
    Ok(Some(owned.into_iter().collect()))
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

fn replace_output(out: &Utf8Path, stage: &Utf8Path) -> anyhow::Result<()> {
    if !out.exists() {
        return std::fs::rename(stage, out)
            .with_context(|| format!("failed to publish staged wiki {stage} to {out}"));
    }
    let backup = unique_path(output_parent(out)?, output_name(out)?, "backup");
    replace_output_with(out, stage, &backup, |from, to| std::fs::rename(from, to))?;
    if let Err(error) = std::fs::remove_dir_all(&backup) {
        eprintln!("warning: published wiki but failed to remove backup {backup}: {error}");
    }
    Ok(())
}

fn replace_output_with(
    out: &Utf8Path,
    stage: &Utf8Path,
    backup: &Utf8Path,
    mut rename: impl FnMut(&Utf8Path, &Utf8Path) -> std::io::Result<()>,
) -> anyhow::Result<()> {
    rename(out, backup)
        .with_context(|| format!("failed to move existing wiki {out} to {backup}"))?;
    if let Err(publish_error) = rename(stage, out) {
        if let Err(rollback_error) = rename(backup, out) {
            bail!(
                "failed to publish staged wiki: {publish_error}; rollback also failed: {rollback_error}; previous output remains at {backup}"
            );
        }
        return Err(publish_error)
            .with_context(|| format!("failed to publish staged wiki {stage}"));
    }
    Ok(())
}

fn create_unique_dir(parent: &Utf8Path, name: &str, purpose: &str) -> anyhow::Result<Utf8PathBuf> {
    loop {
        let path = unique_path(parent, name, purpose);
        match std::fs::create_dir(&path) {
            Ok(()) => return Ok(path),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error).with_context(|| format!("failed to create {path}")),
        }
    }
}

fn unique_path(parent: &Utf8Path, name: &str, purpose: &str) -> Utf8PathBuf {
    let id = UNIQUE_ID.fetch_add(1, Ordering::Relaxed);
    parent.join(format!(".{name}.{purpose}.{}.{id}", std::process::id()))
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

#[cfg(test)]
mod tests {
    use camino::Utf8PathBuf;

    #[test]
    fn failed_replacement_restores_the_previous_output() {
        let dir = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
        let out = root.join("site");
        let stage = root.join("site.stage");
        let backup = root.join("site.backup");
        std::fs::create_dir(&out).unwrap();
        std::fs::write(out.join("index.html"), "old").unwrap();
        std::fs::create_dir(&stage).unwrap();
        std::fs::write(stage.join("index.html"), "new").unwrap();
        let mut calls = 0;

        let error = super::replace_output_with(&out, &stage, &backup, |from, to| {
            calls += 1;
            if calls == 2 {
                Err(std::io::Error::other("injected publication failure"))
            } else {
                std::fs::rename(from, to)
            }
        })
        .unwrap_err();

        assert!(format!("{error:#}").contains("injected publication failure"));
        assert_eq!(
            std::fs::read_to_string(out.join("index.html")).unwrap(),
            "old"
        );
        assert_eq!(
            std::fs::read_to_string(stage.join("index.html")).unwrap(),
            "new"
        );
        assert!(!backup.exists());
    }
}
