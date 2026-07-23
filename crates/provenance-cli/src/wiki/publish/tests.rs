use super::{
    publish, publish_with, replace_output_with, write_page, PublicationOutput, PublishError,
    TransactionPaths, OWNERSHIP_MANIFEST,
};
use crate::wiki::model::{
    CorpusCounts, OrphanReport, PageId, PageKind, RequirementPage, ScopeIndexPage, WikiCorpus,
};
use camino::Utf8PathBuf;
use provenance_core::RequirementStatus;

#[test]
fn publishes_a_complete_fresh_custom_output() {
    let temp = tempfile::tempdir().unwrap();
    let output = Utf8PathBuf::from_path_buf(temp.path().join("wiki")).unwrap();

    let report = publish(&empty_corpus(), PublicationOutput::custom(output.clone())).unwrap();

    assert_eq!(report.page_count, 1);
    assert!(output.join("index.html").is_file());
    assert!(output.join("assets/provenance-wiki.css").is_file());
    assert!(output.join(OWNERSHIP_MANIFEST).is_file());
    assert_no_transaction_artifacts(&output);
}

#[test]
fn adopts_an_empty_custom_directory() {
    let temp = tempfile::tempdir().unwrap();
    let output = utf8(temp.path().join("wiki"));
    std::fs::create_dir(&output).unwrap();

    publish(&empty_corpus(), PublicationOutput::custom(output.clone())).unwrap();

    assert!(output.join(OWNERSHIP_MANIFEST).is_file());
    assert_no_transaction_artifacts(&output);
}

#[test]
fn refuses_a_nonempty_unrecognized_custom_directory_without_mutation() {
    let temp = tempfile::tempdir().unwrap();
    let output = utf8(temp.path().join("wiki"));
    std::fs::create_dir(&output).unwrap();
    std::fs::write(output.join("caller.txt"), "keep me").unwrap();

    let error = publish(&empty_corpus(), PublicationOutput::custom(output.clone())).unwrap_err();

    assert!(matches!(
        error,
        PublishError::CustomOutputUnrecognized { .. }
    ));
    assert_eq!(
        std::fs::read_to_string(output.join("caller.txt")).unwrap(),
        "keep me"
    );
    assert_no_transaction_artifacts(&output);
}

#[test]
fn upgrades_a_recognized_custom_output() {
    let temp = tempfile::tempdir().unwrap();
    let output = utf8(temp.path().join("wiki"));
    publish(&empty_corpus(), PublicationOutput::custom(output.clone())).unwrap();
    std::fs::write(output.join("stale-generated-file"), "old").unwrap();

    publish(&empty_corpus(), PublicationOutput::custom(output.clone())).unwrap();

    assert!(!output.join("stale-generated-file").exists());
    assert!(output.join(OWNERSHIP_MANIFEST).is_file());
}

#[test]
fn default_output_is_generator_owned_even_before_manifests() {
    let temp = tempfile::tempdir().unwrap();
    let output = utf8(temp.path().join("wiki"));
    std::fs::create_dir(&output).unwrap();
    std::fs::write(output.join("legacy-index.html"), "legacy").unwrap();

    publish(
        &empty_corpus(),
        PublicationOutput::generator_owned(output.clone()),
    )
    .unwrap();

    assert!(!output.join("legacy-index.html").exists());
    assert!(output.join(OWNERSHIP_MANIFEST).is_file());
}

#[test]
fn rejects_an_unknown_manifest_version_without_mutation() {
    let temp = tempfile::tempdir().unwrap();
    let output = utf8(temp.path().join("wiki"));
    std::fs::create_dir(&output).unwrap();
    std::fs::write(
        output.join(OWNERSHIP_MANIFEST),
        r#"{"generator":"provenance-wiki","version":99}"#,
    )
    .unwrap();

    let error = publish(&empty_corpus(), PublicationOutput::custom(output.clone())).unwrap_err();

    assert!(matches!(
        error,
        PublishError::UnknownManifestVersion { version: 99, .. }
    ));
    assert_eq!(
        std::fs::read_to_string(output.join(OWNERSHIP_MANIFEST)).unwrap(),
        r#"{"generator":"provenance-wiki","version":99}"#
    );
    assert_no_transaction_artifacts(&output);
}

#[test]
fn rejects_a_malformed_manifest_without_mutation() {
    let temp = tempfile::tempdir().unwrap();
    let output = utf8(temp.path().join("wiki"));
    std::fs::create_dir(&output).unwrap();
    std::fs::write(output.join(OWNERSHIP_MANIFEST), "not json").unwrap();

    let error = publish(&empty_corpus(), PublicationOutput::custom(output.clone())).unwrap_err();

    assert!(matches!(error, PublishError::InvalidManifest { .. }));
    assert_eq!(
        std::fs::read_to_string(output.join(OWNERSHIP_MANIFEST)).unwrap(),
        "not json"
    );
    assert_no_transaction_artifacts(&output);
}

#[test]
fn rejects_a_non_directory_output_root_without_mutation() {
    let temp = tempfile::tempdir().unwrap();
    let output = utf8(temp.path().join("wiki"));
    std::fs::write(&output, "caller bytes").unwrap();

    let error = publish(&empty_corpus(), PublicationOutput::custom(output.clone())).unwrap_err();

    assert!(matches!(error, PublishError::OutputNotDirectory { .. }));
    assert_eq!(std::fs::read_to_string(&output).unwrap(), "caller bytes");
    assert_no_transaction_artifacts(&output);
}

#[cfg(unix)]
#[test]
fn rejects_a_symlink_output_root_without_mutation() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let target = utf8(temp.path().join("target"));
    let output = utf8(temp.path().join("wiki"));
    std::fs::create_dir(&target).unwrap();
    std::fs::write(target.join("caller.txt"), "keep me").unwrap();
    symlink(&target, &output).unwrap();

    let error = publish(&empty_corpus(), PublicationOutput::custom(output.clone())).unwrap_err();

    assert!(matches!(error, PublishError::OutputSymlink { .. }));
    assert!(output.symlink_metadata().unwrap().file_type().is_symlink());
    assert_eq!(
        std::fs::read_to_string(target.join("caller.txt")).unwrap(),
        "keep me"
    );
    assert_no_transaction_artifacts(&output);
}

#[cfg(unix)]
#[test]
fn rejects_a_symlink_lock_without_following_it() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let output = utf8(temp.path().join("wiki"));
    let lock = artifact(&output, "lock");
    let target = utf8(temp.path().join("caller.txt"));
    std::fs::write(&target, "keep me").unwrap();
    symlink(&target, &lock).unwrap();

    let error = publish(&empty_corpus(), PublicationOutput::custom(output.clone())).unwrap_err();

    assert!(matches!(error, PublishError::UnsafeLockPath { .. }));
    assert!(lock.symlink_metadata().unwrap().file_type().is_symlink());
    assert_eq!(std::fs::read_to_string(target).unwrap(), "keep me");
    assert!(!output.exists());
}

#[test]
fn leaves_ambiguous_interruption_artifacts_untouched() {
    let temp = tempfile::tempdir().unwrap();
    let output = utf8(temp.path().join("wiki"));
    let stage = artifact(&output, "stage");
    let backup = artifact(&output, "backup");
    std::fs::create_dir(&stage).unwrap();
    std::fs::create_dir(&backup).unwrap();
    std::fs::write(stage.join("unknown"), "stage bytes").unwrap();
    std::fs::write(backup.join("unknown"), "backup bytes").unwrap();

    let error = publish(&empty_corpus(), PublicationOutput::custom(output.clone())).unwrap_err();

    assert!(matches!(error, PublishError::AmbiguousArtifacts { .. }));
    assert_eq!(
        std::fs::read_to_string(stage.join("unknown")).unwrap(),
        "stage bytes"
    );
    assert_eq!(
        std::fs::read_to_string(backup.join("unknown")).unwrap(),
        "backup bytes"
    );
    assert!(!output.exists());
}

#[test]
fn stage_creation_race_preserves_the_foreign_tree() {
    let temp = tempfile::tempdir().unwrap();
    let output = utf8(temp.path().join("wiki"));
    let stage = artifact(&output, "stage");

    let error = publish_with(
        &empty_corpus(),
        PublicationOutput::custom(output.clone()),
        |path| {
            std::fs::create_dir(path)?;
            std::fs::write(path.join("caller"), "keep me").unwrap();
            std::fs::create_dir(path)
        },
    )
    .unwrap_err();

    assert!(matches!(error, PublishError::Io { .. }));
    assert_eq!(
        std::fs::read_to_string(stage.join("caller")).unwrap(),
        "keep me"
    );
    assert!(!artifact(&output, "lock").exists());
    assert!(!output.exists());
}

#[test]
fn output_replacement_race_preserves_the_intervening_tree() {
    let temp = tempfile::tempdir().unwrap();
    let output = utf8(temp.path().join("wiki"));
    let original = utf8(temp.path().join("original-wiki"));
    publish(&empty_corpus(), PublicationOutput::custom(output.clone())).unwrap();

    let error = publish_with(
        &empty_corpus(),
        PublicationOutput::custom(output.clone()),
        |stage| {
            std::fs::rename(&output, &original)?;
            std::fs::create_dir(&output)?;
            std::fs::copy(
                original.join(OWNERSHIP_MANIFEST),
                output.join(OWNERSHIP_MANIFEST),
            )?;
            std::fs::write(output.join("caller.txt"), "keep me")?;
            std::fs::create_dir(stage)
        },
    )
    .unwrap_err();

    assert!(matches!(
        error,
        PublishError::CleanupFailed { primary, .. }
            if matches!(*primary, PublishError::OutputChanged { .. })
    ));
    assert_eq!(
        std::fs::read_to_string(output.join("caller.txt")).unwrap(),
        "keep me"
    );
    assert!(original.join("index.html").is_file());
    assert!(artifact(&output, "stage").is_dir());
}

#[test]
fn does_not_infer_ownership_from_a_parseable_journal_or_nonce_names() {
    let temp = tempfile::tempdir().unwrap();
    let output = utf8(temp.path().join("wiki"));
    let journal = utf8(temp.path().join(".wiki.publish.json"));
    let nonce_stage = utf8(
        temp.path()
            .join(".wiki.stage.0123456789abcdef0123456789abcdef"),
    );
    let nonce_backup = utf8(
        temp.path()
            .join(".wiki.backup.0123456789abcdef0123456789abcdef"),
    );
    std::fs::write(
        &journal,
        "provenance-wiki-publication-v1\noutput=wiki\nnonce=0123456789abcdef0123456789abcdef\n",
    )
    .unwrap();
    std::fs::create_dir(&nonce_stage).unwrap();
    std::fs::create_dir(&nonce_backup).unwrap();
    std::fs::write(nonce_stage.join("caller"), "stage").unwrap();
    std::fs::write(nonce_backup.join("caller"), "backup").unwrap();
    std::fs::create_dir(&output).unwrap();
    std::fs::write(output.join("caller"), "live").unwrap();

    let error = publish(&empty_corpus(), PublicationOutput::custom(output.clone())).unwrap_err();

    assert!(matches!(
        error,
        PublishError::CustomOutputUnrecognized { .. }
    ));
    assert_eq!(
        std::fs::read_to_string(output.join("caller")).unwrap(),
        "live"
    );
    assert!(journal.is_file());
    assert_eq!(
        std::fs::read_to_string(nonce_stage.join("caller")).unwrap(),
        "stage"
    );
    assert_eq!(
        std::fs::read_to_string(nonce_backup.join("caller")).unwrap(),
        "backup"
    );
}

#[test]
fn a_returned_install_failure_restores_the_previous_output() {
    let temp = tempfile::tempdir().unwrap();
    let output = utf8(temp.path().join("wiki"));
    let paths = TransactionPaths::new(&output).unwrap();
    std::fs::create_dir(&output).unwrap();
    std::fs::write(output.join("generation"), "old").unwrap();
    std::fs::create_dir(&paths.stage).unwrap();
    std::fs::write(paths.stage.join("generation"), "new").unwrap();
    let mut rename_count = 0;

    let error = replace_output_with(
        &output,
        &paths,
        |_, _| {
            rename_count += 1;
            if rename_count == 2 {
                Err(std::io::Error::other("injected second rename failure"))
            } else {
                Ok(())
            }
        },
        |_| Ok(()),
    )
    .unwrap_err();

    assert!(matches!(error, PublishError::ReplacementRolledBack { .. }));
    assert_eq!(
        std::fs::read_to_string(output.join("generation")).unwrap(),
        "old"
    );
    assert_eq!(
        std::fs::read_to_string(paths.stage.join("generation")).unwrap(),
        "new"
    );
    assert!(!paths.backup.exists());
}

#[test]
fn output_appearance_between_renames_preserves_every_tree() {
    let temp = tempfile::tempdir().unwrap();
    let output = utf8(temp.path().join("wiki"));
    let paths = TransactionPaths::new(&output).unwrap();
    std::fs::create_dir(&output).unwrap();
    std::fs::write(output.join("generation"), "old").unwrap();
    std::fs::create_dir(&paths.stage).unwrap();
    std::fs::write(paths.stage.join("generation"), "new").unwrap();
    let mut rename_count = 0;

    let error = replace_output_with(
        &output,
        &paths,
        |_, _| {
            rename_count += 1;
            if rename_count == 2 {
                std::fs::create_dir(&output)?;
            }
            Ok(())
        },
        |_| Ok(()),
    )
    .unwrap_err();

    assert!(matches!(error, PublishError::RollbackFailed { .. }));
    assert!(output.is_dir());
    assert_eq!(
        std::fs::read_to_string(paths.backup.join("generation")).unwrap(),
        "old"
    );
    assert_eq!(
        std::fs::read_to_string(paths.stage.join("generation")).unwrap(),
        "new"
    );
}

#[test]
fn committed_cleanup_failure_is_a_warning_not_a_returned_failure() {
    let temp = tempfile::tempdir().unwrap();
    let output = utf8(temp.path().join("wiki"));
    let paths = TransactionPaths::new(&output).unwrap();
    std::fs::create_dir(&output).unwrap();
    std::fs::write(output.join("generation"), "old").unwrap();
    std::fs::create_dir(&paths.stage).unwrap();
    std::fs::write(paths.stage.join("generation"), "new").unwrap();

    let warnings = replace_output_with(
        &output,
        &paths,
        |_, _| Ok(()),
        |_path| Err(std::io::Error::other("injected cleanup failure")),
    )
    .unwrap();

    assert_eq!(warnings.len(), 1);
    assert_eq!(
        std::fs::read_to_string(output.join("generation")).unwrap(),
        "new"
    );
    assert_eq!(
        std::fs::read_to_string(paths.backup.join("generation")).unwrap(),
        "old"
    );
}

#[test]
fn backup_cleanup_race_preserves_the_replacement_tree() {
    let temp = tempfile::tempdir().unwrap();
    let output = utf8(temp.path().join("wiki"));
    let paths = TransactionPaths::new(&output).unwrap();
    let displaced_backup = utf8(temp.path().join("displaced-backup"));
    std::fs::create_dir(&output).unwrap();
    std::fs::write(output.join("generation"), "old").unwrap();
    std::fs::create_dir(&paths.stage).unwrap();
    std::fs::write(paths.stage.join("generation"), "new").unwrap();

    let warnings = replace_output_with(
        &output,
        &paths,
        |_, _| Ok(()),
        |path| {
            std::fs::rename(path, &displaced_backup)?;
            std::fs::create_dir(path)?;
            std::fs::write(path.join("caller"), "keep me")
        },
    )
    .unwrap();

    assert_eq!(warnings.len(), 1);
    assert_eq!(
        std::fs::read_to_string(paths.backup.join("caller")).unwrap(),
        "keep me"
    );
    assert_eq!(
        std::fs::read_to_string(displaced_backup.join("generation")).unwrap(),
        "old"
    );
}

#[test]
fn rejects_a_route_that_would_escape_the_stage() {
    let temp = tempfile::tempdir().unwrap();
    let stage = utf8(temp.path().join("stage"));
    std::fs::create_dir(&stage).unwrap();

    let error = write_page(&stage, "/../escaped/", "caller overwrite").unwrap_err();

    assert!(matches!(error, PublishError::InvalidRoute { .. }));
    assert!(!utf8(temp.path().join("escaped/index.html")).exists());
}

#[test]
fn staging_failure_preserves_the_previous_complete_output() {
    let temp = tempfile::tempdir().unwrap();
    let output = utf8(temp.path().join("wiki"));
    publish(&empty_corpus(), PublicationOutput::custom(output.clone())).unwrap();
    let previous_index = std::fs::read(output.join("index.html")).unwrap();
    let mut invalid = empty_corpus();
    invalid.requirements.push(RequirementPage {
        id: PageId::new(PageKind::Requirement, "../escape"),
        title: "unsafe".to_string(),
        status: RequirementStatus::Active,
        statement: "unsafe".to_string(),
        description: None,
        fog: None,
        domain_id: None,
        back_link: None,
        lineage: Vec::new(),
        decisions: Vec::new(),
        produced_rules: Vec::new(),
        children: Vec::new(),
        siblings: Vec::new(),
        sources: Vec::new(),
        gaps: Vec::new(),
        threads: Vec::new(),
    });

    let error = publish(&invalid, PublicationOutput::custom(output.clone())).unwrap_err();

    assert!(matches!(
        error,
        PublishError::CleanupFailed { primary, .. }
            if matches!(*primary, PublishError::InvalidRoute { .. })
    ));
    assert_eq!(
        std::fs::read(output.join("index.html")).unwrap(),
        previous_index
    );
    assert!(artifact(&output, "stage").is_dir());
    assert!(!utf8(temp.path().join("escape")).exists());
}

fn empty_corpus() -> WikiCorpus {
    WikiCorpus {
        scope: "default".to_string(),
        index: ScopeIndexPage {
            id: PageId::new(PageKind::ScopeIndex, "default"),
            scope: "default".to_string(),
            title: "Provenance Wiki".to_string(),
            counts: CorpusCounts::default(),
            roots: Vec::new(),
            gaps: Vec::new(),
            orphans: OrphanReport::default(),
        },
        requirements: Vec::new(),
        resolutions: Vec::new(),
        rules: Vec::new(),
        sources: Vec::new(),
    }
}

fn assert_no_transaction_artifacts(output: &camino::Utf8Path) {
    for role in ["lock", "stage", "backup"] {
        assert!(!artifact(output, role).exists());
    }
}

fn artifact(output: &camino::Utf8Path, role: &str) -> Utf8PathBuf {
    let parent = output.parent().unwrap();
    let leaf = output.file_name().unwrap();
    parent.join(format!(".{leaf}.provenance-wiki.{role}"))
}

fn utf8(path: std::path::PathBuf) -> Utf8PathBuf {
    Utf8PathBuf::from_path_buf(path).unwrap()
}
