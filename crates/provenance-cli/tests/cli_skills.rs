use assert_cmd::Command;
use predicates::prelude::*;
use std::collections::BTreeSet;
use std::path::PathBuf;

#[test]
fn skills_list_and_show_embedded_skill_files() {
    let skills = workspace_skill_files();

    let output = Command::cargo_bin("provenance")
        .unwrap()
        .args(["skills", "list", "--format", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let listed: Vec<serde_json::Value> = serde_json::from_slice(&output).unwrap();
    let listed_names = listed
        .iter()
        .map(|skill| skill["name"].as_str().unwrap().to_string())
        .collect::<BTreeSet<_>>();

    assert_eq!(listed_names, skills.keys().cloned().collect());
    assert!(listed.iter().all(|skill| skill["description"]
        .as_str()
        .is_some_and(|description| !description.is_empty())));

    for (name, contents) in skills {
        Command::cargo_bin("provenance")
            .unwrap()
            .args(["skills", "show", &name])
            .assert()
            .success()
            .stdout(contents);
    }
}

#[test]
fn embedded_skills_include_turn_based_provenance_shaping_skill() {
    Command::cargo_bin("provenance")
        .unwrap()
        .args(["skills", "list", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""name": "provenance-shaping""#));

    Command::cargo_bin("provenance")
        .unwrap()
        .args(["skills", "show", "provenance-shaping"])
        .assert()
        .success()
        .stdout(predicate::str::contains("LAND-AS-YOU-GO"))
        .stdout(predicate::str::contains("Chart"))
        .stdout(predicate::str::contains("Work"));
}

#[test]
fn skills_install_default_writes_canonical_files_and_relative_claude_symlinks() {
    let dir = tempfile::tempdir().unwrap();
    let skill = "provenance-fork-tournament";

    Command::cargo_bin("provenance")
        .unwrap()
        .current_dir(dir.path())
        .args(["skills", "install", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""link_mode": "symlink""#));

    let canonical = dir
        .path()
        .join(".agents/skills")
        .join(skill)
        .join("SKILL.md");
    let canonical_contents = std::fs::read_to_string(&canonical).unwrap();
    assert!(canonical_contents.starts_with("---\nname: provenance-fork-tournament"));
    assert!(canonical_contents.contains(&format!(
        "Installed by provenance {}",
        env!("CARGO_PKG_VERSION")
    )));
    assert!(canonical_contents.contains("content hash fnv1a64:"));

    let link = dir.path().join(".claude/skills").join(skill);
    let link_metadata = std::fs::symlink_metadata(&link).unwrap();
    assert!(link_metadata.file_type().is_symlink());
    assert_eq!(
        std::fs::read_link(&link).unwrap(),
        PathBuf::from("../../.agents/skills/provenance-fork-tournament")
    );
    assert_eq!(
        std::fs::read_to_string(link.join("SKILL.md")).unwrap(),
        canonical_contents
    );
}

#[test]
fn skills_install_copy_flag_copies_claude_skills_instead_of_symlinking() {
    let dir = tempfile::tempdir().unwrap();
    let skill = "provenance-swarm-backtrace";

    Command::cargo_bin("provenance")
        .unwrap()
        .current_dir(dir.path())
        .args(["skills", "install", "--copy", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""link_mode": "copy""#));

    let canonical = dir
        .path()
        .join(".agents/skills")
        .join(skill)
        .join("SKILL.md");
    let copied = dir
        .path()
        .join(".claude/skills")
        .join(skill)
        .join("SKILL.md");
    assert!(canonical.exists());
    assert!(copied.exists());
    assert!(!std::fs::symlink_metadata(copied.parent().unwrap())
        .unwrap()
        .file_type()
        .is_symlink());
    assert_eq!(
        std::fs::read_to_string(copied).unwrap(),
        std::fs::read_to_string(canonical).unwrap()
    );
}

#[test]
#[cfg(unix)]
fn skills_install_copy_replaces_own_symlink_but_foreign_symlink_requires_force() {
    let dir = tempfile::tempdir().unwrap();
    let skill = "provenance-shaping";
    let link = dir.path().join(".claude/skills").join(skill);

    // Default install, then --copy: our own canonical symlink is replaced
    // with a real directory without needing --force.
    Command::cargo_bin("provenance")
        .unwrap()
        .current_dir(dir.path())
        .args(["skills", "install", "--format", "json"])
        .assert()
        .success();
    Command::cargo_bin("provenance")
        .unwrap()
        .current_dir(dir.path())
        .args(["skills", "install", "--copy", "--format", "json"])
        .assert()
        .success();
    let metadata = std::fs::symlink_metadata(&link).unwrap();
    assert!(metadata.is_dir());
    assert!(!metadata.file_type().is_symlink());

    // A foreign symlink is not silently destroyed.
    std::fs::remove_dir_all(&link).unwrap();
    std::os::unix::fs::symlink("../../elsewhere", &link).unwrap();
    Command::cargo_bin("provenance")
        .unwrap()
        .current_dir(dir.path())
        .args(["skills", "install", "--copy", "--format", "json"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("rerun with --force"));
    assert_eq!(
        std::fs::read_link(&link).unwrap(),
        PathBuf::from("../../elsewhere")
    );

    Command::cargo_bin("provenance")
        .unwrap()
        .current_dir(dir.path())
        .args(["skills", "install", "--copy", "--force", "--format", "json"])
        .assert()
        .success();
    assert!(std::fs::symlink_metadata(&link).unwrap().is_dir());
}

#[test]
fn skills_install_is_idempotent_and_requires_force_for_canonical_drift() {
    let dir = tempfile::tempdir().unwrap();
    let installed = dir
        .path()
        .join(".agents/skills/provenance-fork-tournament/SKILL.md");

    Command::cargo_bin("provenance")
        .unwrap()
        .current_dir(dir.path())
        .args(["skills", "install", "--format", "json"])
        .assert()
        .success();

    Command::cargo_bin("provenance")
        .unwrap()
        .current_dir(dir.path())
        .args(["skills", "install", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""status": "unchanged""#));

    std::fs::write(&installed, "local edit\n").unwrap();
    Command::cargo_bin("provenance")
        .unwrap()
        .current_dir(dir.path())
        .args(["skills", "install", "--format", "json"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("exists and differs"));

    Command::cargo_bin("provenance")
        .unwrap()
        .current_dir(dir.path())
        .args(["skills", "install", "--force", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""status": "updated""#));
}

#[test]
fn skills_install_global_uses_home_agents_and_claude_skill_dirs() {
    let home = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();

    Command::cargo_bin("provenance")
        .unwrap()
        .current_dir(cwd.path())
        .env("HOME", home.path())
        .args([
            "skills", "install", "--global", "--copy", "--format", "json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""global": true"#))
        .stdout(predicate::str::contains(r#""link_mode": "copy""#));

    assert!(home
        .path()
        .join(".agents/skills/provenance-grounded-writing/SKILL.md")
        .exists());
    assert!(home
        .path()
        .join(".claude/skills/provenance-grounded-writing/SKILL.md")
        .exists());
    assert!(!cwd
        .path()
        .join(".agents/skills/provenance-grounded-writing/SKILL.md")
        .exists());
}

#[test]
fn init_does_not_write_an_agents_md_skills_section() {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("repo");

    init_repo(&repo);

    let agents = repo.join("AGENTS.md");
    if agents.exists() {
        let contents = std::fs::read_to_string(agents).unwrap();
        assert!(!contents.contains("<!-- BEGIN PROVENANCE SKILLS -->"));
    }
}

#[test]
fn prime_reports_skill_install_status_and_install_command() {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("repo");
    init_repo(&repo);

    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "prime",
            "--repo",
            repo.to_str().unwrap(),
            "--scope",
            "default",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""skills""#))
        .stdout(predicate::str::contains(r#""installed": false"#))
        .stdout(predicate::str::contains("provenance skills install"));

    Command::cargo_bin("provenance")
        .unwrap()
        .current_dir(&repo)
        .args(["skills", "install", "--copy"])
        .assert()
        .success();

    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "prime",
            "--repo",
            repo.to_str().unwrap(),
            "--scope",
            "default",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""installed": true"#));
}

#[test]
fn install_status_uses_canonical_agents_skill_files_as_source_of_truth() {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("repo");
    init_repo(&repo);

    Command::cargo_bin("provenance")
        .unwrap()
        .current_dir(&repo)
        .args(["skills", "install", "--copy"])
        .assert()
        .success();
    std::fs::remove_file(repo.join(".agents/skills/provenance-fork-tournament/SKILL.md")).unwrap();
    assert!(repo
        .join(".claude/skills/provenance-fork-tournament/SKILL.md")
        .exists());

    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "prime",
            "--repo",
            repo.to_str().unwrap(),
            "--scope",
            "default",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""installed": false"#))
        .stdout(predicate::str::contains("provenance-fork-tournament"));
}

#[test]
fn shaping_and_ideation_commands_emit_suppressible_skill_install_hint() {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("repo");
    init_repo(&repo);

    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "questions",
            "list",
            "--repo",
            repo.to_str().unwrap(),
            "--scope",
            "default",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains(
            "hint: provenance skills are not installed; run `provenance skills install`",
        ));

    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "proposals",
            "list",
            "--repo",
            repo.to_str().unwrap(),
            "--scope",
            "default",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains(
            "hint: provenance skills are not installed; run `provenance skills install`",
        ));

    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "--quiet",
            "questions",
            "list",
            "--repo",
            repo.to_str().unwrap(),
            "--scope",
            "default",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

fn init_repo(repo: &std::path::Path) {
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "init",
            "--path",
            repo.to_str().unwrap(),
            "--scope",
            "default",
            "--path-prefix",
            ".",
        ])
        .assert()
        .success();
}

fn workspace_skill_files() -> std::collections::BTreeMap<String, String> {
    let workspace = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let mut skills = std::collections::BTreeMap::new();
    for entry in std::fs::read_dir(workspace.join("skills")).unwrap() {
        let entry = entry.unwrap();
        if !entry.file_type().unwrap().is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let contents = std::fs::read_to_string(entry.path().join("SKILL.md")).unwrap();
        skills.insert(name, contents);
    }
    skills
}
