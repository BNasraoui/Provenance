use super::{OutputPolicy, PublishError, GENERATOR, MANIFEST_VERSION, OWNERSHIP_MANIFEST};
use camino::Utf8Path;
use serde::{Deserialize, Serialize};
use std::io::Read;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct OwnershipManifest<'a> {
    #[serde(borrow)]
    pub generator: std::borrow::Cow<'a, str>,
    pub version: u32,
}

pub(super) fn validate_output_handle(
    directory: Option<std::fs::File>,
    output: &Utf8Path,
    policy: OutputPolicy,
) -> Result<(), PublishError> {
    let Some(directory) = directory else {
        return Ok(());
    };
    let manifest_path = output.join(OWNERSHIP_MANIFEST);
    let mut options = fs_at::OpenOptions::default();
    options.read(true).follow(false);
    match options.open_at(&directory, OWNERSHIP_MANIFEST) {
        Ok(mut file) => {
            if !file
                .metadata()
                .map_err(|error| {
                    PublishError::io("inspect ownership marker", &manifest_path, error)
                })?
                .is_file()
            {
                return Err(PublishError::InvalidManifest {
                    path: manifest_path,
                    detail: "marker is not a regular file".to_string(),
                });
            }
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes)
                .map_err(|error| PublishError::io("read", &manifest_path, error))?;
            validate_manifest_bytes(&manifest_path, &bytes)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            if policy == OutputPolicy::GeneratorOwned || directory_is_empty_at(directory, output)? {
                Ok(())
            } else {
                Err(PublishError::CustomOutputUnrecognized {
                    path: output.to_path_buf(),
                })
            }
        }
        Err(error) => Err(PublishError::io(
            "inspect ownership marker",
            &manifest_path,
            error,
        )),
    }
}

fn validate_manifest_bytes(path: &Utf8Path, bytes: &[u8]) -> Result<(), PublishError> {
    let manifest: OwnershipManifest<'_> =
        serde_json::from_slice(bytes).map_err(|error| PublishError::InvalidManifest {
            path: path.to_path_buf(),
            detail: error.to_string(),
        })?;
    if manifest.generator != GENERATOR {
        return Err(PublishError::InvalidManifest {
            path: path.to_path_buf(),
            detail: format!("unknown generator {:?}", manifest.generator),
        });
    }
    if manifest.version != MANIFEST_VERSION {
        return Err(PublishError::UnknownManifestVersion {
            path: path.to_path_buf(),
            version: manifest.version,
        });
    }
    Ok(())
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
