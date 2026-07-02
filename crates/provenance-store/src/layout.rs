use camino::{Utf8Path, Utf8PathBuf};

#[derive(Debug, Clone)]
pub struct ProvenanceLayout {
    root: Utf8PathBuf,
}

impl ProvenanceLayout {
    pub fn new(root: impl Into<Utf8PathBuf>) -> Self {
        Self { root: root.into() }
    }
    pub fn provenance_dir(&self) -> Utf8PathBuf {
        self.root.join(".provenance")
    }
    pub fn state_dir(&self) -> Utf8PathBuf {
        self.provenance_dir().join("state")
    }
    pub fn manifest_path(&self) -> Utf8PathBuf {
        self.state_dir().join("manifest.json")
    }
    pub fn scopes_dir(&self) -> Utf8PathBuf {
        self.state_dir().join("scopes")
    }
    pub fn edges_dir(&self) -> Utf8PathBuf {
        self.state_dir().join("edges")
    }
    pub fn cache_dir(&self) -> Utf8PathBuf {
        self.provenance_dir().join("cache")
    }
    pub fn cache_db_path(&self) -> Utf8PathBuf {
        self.cache_dir().join("provenance.db")
    }
}

pub fn locate_repo_root(start: &Utf8Path) -> anyhow::Result<Utf8PathBuf> {
    for candidate in start.ancestors() {
        if candidate.join(".git").exists()
            || candidate.join(".provenance/state/manifest.json").exists()
        {
            return Ok(candidate.to_path_buf());
        }
    }
    anyhow::bail!("could not locate repository root from {start}")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn layout_paths_are_under_provenance_directory() {
        let layout = ProvenanceLayout::new("/tmp/repo");
        assert_eq!(
            layout.manifest_path().as_str(),
            "/tmp/repo/.provenance/state/manifest.json"
        );
        assert_eq!(
            layout.cache_db_path().as_str(),
            "/tmp/repo/.provenance/cache/provenance.db"
        );
    }
}
