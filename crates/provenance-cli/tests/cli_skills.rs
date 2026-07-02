use assert_cmd::Command;
use predicates::prelude::*;
use std::collections::BTreeSet;

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
fn embedded_skills_include_turn_based_shaping_skill() {
    Command::cargo_bin("provenance")
        .unwrap()
        .args(["skills", "list", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""name": "shaping""#));

    Command::cargo_bin("provenance")
        .unwrap()
        .args(["skills", "show", "shaping"])
        .assert()
        .success()
        .stdout(predicate::str::contains("LAND-AS-YOU-GO"))
        .stdout(predicate::str::contains("Chart"))
        .stdout(predicate::str::contains("Work"));
}

#[test]
fn skills_install_claude_target_is_idempotent_and_requires_force_for_drift() {
    let dir = tempfile::tempdir().unwrap();

    Command::cargo_bin("provenance")
        .unwrap()
        .current_dir(dir.path())
        .args([
            "skills", "install", "--target", "claude", "--format", "json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""target": "claude""#));

    let installed = dir.path().join(".claude/skills/fork-tournament/SKILL.md");
    let installed_contents = std::fs::read_to_string(&installed).unwrap();
    assert!(installed_contents.starts_with("---\nname: fork-tournament"));
    assert!(installed_contents.contains(&format!(
        "Installed by provenance {}",
        env!("CARGO_PKG_VERSION")
    )));
    assert!(installed_contents.contains("content hash fnv1a64:"));

    Command::cargo_bin("provenance")
        .unwrap()
        .current_dir(dir.path())
        .args([
            "skills", "install", "--target", "claude", "--format", "json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""status": "unchanged""#));

    std::fs::write(&installed, "local edit\n").unwrap();
    Command::cargo_bin("provenance")
        .unwrap()
        .current_dir(dir.path())
        .args([
            "skills", "install", "--target", "claude", "--format", "json",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("exists and differs"));

    Command::cargo_bin("provenance")
        .unwrap()
        .current_dir(dir.path())
        .args([
            "skills", "install", "--target", "claude", "--force", "--format", "json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""status": "updated""#));
}

#[test]
fn skills_install_opencode_target_writes_agents_skills() {
    let dir = tempfile::tempdir().unwrap();

    Command::cargo_bin("provenance")
        .unwrap()
        .current_dir(dir.path())
        .args([
            "skills", "install", "--target", "opencode", "--format", "json",
        ])
        .assert()
        .success();

    assert!(dir
        .path()
        .join(".agents/skills/fork-tournament/SKILL.md")
        .exists());
    assert!(dir
        .path()
        .join(".agents/skills/swarm-backtrace/SKILL.md")
        .exists());
}

#[test]
fn skills_install_agents_md_appends_managed_section() {
    let dir = tempfile::tempdir().unwrap();
    let agents = dir.path().join("AGENTS.md");
    std::fs::write(&agents, "# Existing Instructions\n").unwrap();

    Command::cargo_bin("provenance")
        .unwrap()
        .current_dir(dir.path())
        .args([
            "skills",
            "install",
            "--target",
            "agents-md",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""target": "agents-md""#));

    let installed = std::fs::read_to_string(&agents).unwrap();
    assert!(installed.starts_with("# Existing Instructions\n"));
    assert!(installed.contains("<!-- BEGIN PROVENANCE SKILLS -->"));
    assert!(installed.contains(&format!(
        "Installed by provenance {}",
        env!("CARGO_PKG_VERSION")
    )));
    assert!(installed.contains("## Skill: fork-tournament"));
    assert!(installed.contains("# Fork tournament (`prototype`)"));
    assert!(installed.contains("## Skill: swarm-backtrace"));
    assert!(installed.contains("# Swarm backtrace"));

    Command::cargo_bin("provenance")
        .unwrap()
        .current_dir(dir.path())
        .args([
            "skills",
            "install",
            "--target",
            "agents-md",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""status": "unchanged""#));
}

#[test]
fn init_injects_provenance_agents_md_section() {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("repo");

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

    let agents = std::fs::read_to_string(repo.join("AGENTS.md")).unwrap();
    assert!(agents.contains("<!-- BEGIN PROVENANCE SKILLS -->"));
    assert!(agents.contains(
        "Before shaping or backtrace work, run `provenance skills install --target agents-md` if skills are absent."
    ));
    assert!(agents.contains("## Skill: fork-tournament"));
    assert!(agents.contains("## Skill: swarm-backtrace"));
}

#[test]
fn prime_reports_skill_install_status_and_install_command() {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("repo");
    init_repo(&repo);
    std::fs::remove_file(repo.join("AGENTS.md")).unwrap();

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
        .stdout(predicate::str::contains(
            "provenance skills install --target agents-md",
        ));

    Command::cargo_bin("provenance")
        .unwrap()
        .current_dir(&repo)
        .args(["skills", "install", "--target", "agents-md"])
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
fn shaping_and_ideation_commands_emit_suppressible_skill_install_hint() {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("repo");
    init_repo(&repo);
    std::fs::remove_file(repo.join("AGENTS.md")).unwrap();

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
            "hint: provenance skills are not installed; run `provenance skills install --target agents-md`",
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
            "hint: provenance skills are not installed; run `provenance skills install --target agents-md`",
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
