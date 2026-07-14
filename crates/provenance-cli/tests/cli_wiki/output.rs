use super::support::{init_repo, seed_full_state};
use assert_cmd::Command;

#[test]
fn wiki_build_default_format_prints_a_concise_summary_not_a_page_dump() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("repo").to_string_lossy().to_string();
    let out = dir.path().join("site");
    seed_full_state(dir.path(), &repo);
    let stdout = Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "wiki",
            "build",
            "--repo",
            &repo,
            "--out",
            &out.to_string_lossy(),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(stdout).unwrap();
    assert!(!stdout.contains("\"pages\""), "{stdout}");
    assert!(!stdout.contains("\"route\""), "{stdout}");
    assert!(stdout.contains("8 pages"), "{stdout}");
    assert!(stdout.contains("wiki serve"), "{stdout}");
    assert!(stdout.contains(out.to_string_lossy().as_ref()), "{stdout}");
}

#[test]
fn wiki_build_defaults_output_to_the_provenance_wiki_dir_and_gitignores_it() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("repo").to_string_lossy().to_string();
    init_repo(&repo);
    Command::cargo_bin("provenance")
        .unwrap()
        .args(["wiki", "build", "--repo", &repo, "--format", "json"])
        .assert()
        .success()
        .stdout(predicates::str::contains(".provenance/wiki"));
    let default_out = std::path::Path::new(&repo).join(".provenance/wiki");
    assert!(default_out.join("index.html").exists());
    let gitignore =
        std::fs::read_to_string(std::path::Path::new(&repo).join(".gitignore")).unwrap();
    assert!(gitignore
        .lines()
        .any(|line| line.trim() == ".provenance/wiki/"));
}

#[test]
fn wiki_build_with_an_explicit_out_does_not_touch_gitignore() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("repo").to_string_lossy().to_string();
    let out = dir.path().join("site");
    init_repo(&repo);
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "wiki",
            "build",
            "--repo",
            &repo,
            "--out",
            &out.to_string_lossy(),
            "--format",
            "json",
        ])
        .assert()
        .success();
    assert!(!std::path::Path::new(&repo).join(".gitignore").exists());
}
