use super::*;

#[cfg(unix)]
#[test]
fn staged_child_symlink_cannot_redirect_generated_writes() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let output = utf8(temp.path().join("wiki"));
    let external = utf8(temp.path().join("external"));
    std::fs::create_dir(&external).unwrap();

    let error = publish_with(
        &empty_corpus(),
        PublicationOutput::custom(output.clone()),
        |stage| symlink(&external, stage.join("assets")),
    )
    .unwrap_err();

    assert!(matches!(
        error,
        PublishError::CleanupFailed { primary, .. }
            if matches!(*primary, PublishError::Io { .. })
    ));
    assert!(!external.join("provenance-wiki.css").exists());
    assert!(!output.exists());
}

#[cfg(windows)]
#[test]
fn staged_child_reparse_point_cannot_redirect_generated_writes() {
    use std::os::windows::fs::symlink_dir;

    let temp = tempfile::tempdir().unwrap();
    let output = utf8(temp.path().join("wiki"));
    let external = utf8(temp.path().join("external"));
    std::fs::create_dir(&external).unwrap();

    let result = publish_with(
        &empty_corpus(),
        PublicationOutput::custom(output.clone()),
        |stage| symlink_dir(&external, stage.join("assets")),
    );
    let error = match result {
        Err(error) => error,
        Ok(_) => panic!("publication followed a staged reparse point"),
    };

    assert!(matches!(
        error,
        PublishError::CleanupFailed { primary, .. }
            if matches!(*primary, PublishError::Io { .. })
    ));
    assert!(!external.join("provenance-wiki.css").exists());
    assert!(!output.exists());
}

#[test]
fn replaced_stage_is_not_installed() {
    let temp = tempfile::tempdir().unwrap();
    let output = utf8(temp.path().join("wiki"));
    let displaced = utf8(temp.path().join("displaced-stage"));

    let error = publish_with(
        &empty_corpus(),
        PublicationOutput::custom(output.clone()),
        |stage| {
            std::fs::rename(stage, &displaced)?;
            std::fs::create_dir(stage)?;
            std::fs::write(stage.join("caller"), "keep me")
        },
    )
    .unwrap_err();

    assert!(matches!(
        error,
        PublishError::CleanupFailed { primary, .. }
            if matches!(*primary, PublishError::OutputChanged { .. })
    ));
    assert_eq!(
        std::fs::read_to_string(artifact(&output, "stage").join("caller")).unwrap(),
        "keep me"
    );
    assert!(displaced.join("index.html").is_file());
    assert!(!output.exists());
}

#[test]
fn empty_replacement_stage_is_preserved() {
    let temp = tempfile::tempdir().unwrap();
    let output = utf8(temp.path().join("wiki"));
    let displaced = utf8(temp.path().join("displaced-stage"));
    let stage = artifact(&output, "stage");

    let error = publish_with(
        &empty_corpus(),
        PublicationOutput::custom(output.clone()),
        |stage| {
            std::fs::rename(stage, &displaced)?;
            std::fs::create_dir(stage)
        },
    )
    .unwrap_err();

    assert!(matches!(error, PublishError::CleanupFailed { .. }));
    assert!(stage.is_dir());
    assert!(std::fs::read_dir(stage).unwrap().next().is_none());
    assert!(displaced.join("index.html").is_file());
    assert!(!output.exists());
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
