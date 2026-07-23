use super::{OutputPolicy, PublishError, GENERATOR, MANIFEST_VERSION, OWNERSHIP_MANIFEST};
use camino::Utf8Path;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct OwnershipManifest<'a> {
    #[serde(borrow)]
    pub generator: std::borrow::Cow<'a, str>,
    pub version: u32,
}

pub(super) fn validate_output(output: &Utf8Path, policy: OutputPolicy) -> Result<(), PublishError> {
    let metadata = match output.symlink_metadata() {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(PublishError::io("inspect wiki output", output, error)),
    };
    if metadata.file_type().is_symlink() {
        return Err(PublishError::OutputSymlink {
            path: output.to_path_buf(),
        });
    }
    if !metadata.is_dir() {
        return Err(PublishError::OutputNotDirectory {
            path: output.to_path_buf(),
        });
    }

    let manifest_path = output.join(OWNERSHIP_MANIFEST);
    match manifest_path.symlink_metadata() {
        Ok(metadata) => {
            if !metadata.is_file() || metadata.file_type().is_symlink() {
                return Err(PublishError::InvalidManifest {
                    path: manifest_path,
                    detail: "marker is not a regular file".to_string(),
                });
            }
            validate_manifest(&manifest_path)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            if policy == OutputPolicy::GeneratorOwned || directory_is_empty(output)? {
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

fn validate_manifest(path: &Utf8Path) -> Result<(), PublishError> {
    let bytes = std::fs::read(path).map_err(|error| PublishError::io("read", path, error))?;
    let manifest: OwnershipManifest<'_> =
        serde_json::from_slice(&bytes).map_err(|error| PublishError::InvalidManifest {
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

fn directory_is_empty(path: &Utf8Path) -> Result<bool, PublishError> {
    let mut entries = std::fs::read_dir(path)
        .map_err(|error| PublishError::io("read custom output directory", path, error))?;
    Ok(entries.next().is_none())
}
