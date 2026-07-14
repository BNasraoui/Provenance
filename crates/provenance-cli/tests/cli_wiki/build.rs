use super::support::seed_full_state;
use assert_cmd::Command;

#[test]
fn wiki_build_writes_static_pages_and_stylesheet() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("repo");
    let repo = repo.to_string_lossy().to_string();
    let out = dir.path().join("site");
    seed_full_state(dir.path(), &repo);
    std::fs::create_dir_all(out.join("assets")).unwrap();
    std::fs::write(out.join("assets/search-index.json"), "stale").unwrap();
    std::fs::create_dir_all(out.join("customer/nested")).unwrap();
    std::fs::write(out.join("customer/nested/notes.txt"), "keep me").unwrap();

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
        .success()
        .stdout(predicates::str::contains(r#""status": "ok""#))
        .stdout(predicates::str::contains(r#""scope": "default""#))
        .stdout(predicates::str::contains(
            r#""route": "/requirements/req_sah/""#,
        ));

    let index = std::fs::read_to_string(out.join("index.html")).unwrap();
    assert!(index.contains("Provenance Wiki"), "{index}");
    assert!(index.contains("href=\"/requirements/req_sah/\""), "{index}");
    assert!(index.contains("href=\"/topics/\""), "{index}");
    assert!(index.contains("href=\"/search/\""), "{index}");

    let topics = std::fs::read_to_string(out.join("topics/index.html")).unwrap();
    assert!(topics.contains("Care delivery"), "{topics}");
    assert!(
        topics.contains("href=\"/requirements/req_sah/\""),
        "{topics}"
    );
    assert!(topics.contains("href=\"/rules/rule_sah_001/\""), "{topics}");

    let search = std::fs::read_to_string(out.join("search/index.html")).unwrap();
    assert!(search.contains("id=\"wiki-search\""), "{search}");
    assert!(
        search.contains("Support at Home shall be traceable"),
        "{search}"
    );
    assert!(search.contains("Draft rule shall stay draft"), "{search}");
    assert_eq!(
        std::fs::read_to_string(out.join("assets/search-index.json")).unwrap(),
        "stale",
        "an unowned legacy pathname must not be deleted"
    );
    assert_eq!(
        std::fs::read_to_string(out.join("customer/nested/notes.txt")).unwrap(),
        "keep me",
        "arbitrary caller output must survive publication"
    );

    let stylesheet = std::fs::read_to_string(out.join("assets/provenance-wiki.css")).unwrap();
    assert!(stylesheet.contains("--pv-"), "stylesheet missing tokens");
    let requirement = std::fs::read_to_string(out.join("requirements/req_sah/index.html")).unwrap();
    assert!(
        requirement.contains("Support at Home shall be traceable"),
        "{requirement}"
    );
    assert!(requirement.contains("SAH-001"), "{requirement}");
    assert!(
        requirement.contains("href=\"/assets/provenance-wiki.css\""),
        "{requirement}"
    );
    let rule = std::fs::read_to_string(out.join("rules/rule_sah_001/index.html")).unwrap();
    assert!(rule.contains("Example-API-main/src/example.php"), "{rule}");
    let gapped = std::fs::read_to_string(out.join("requirements/req_gap/index.html")).unwrap();
    assert!(gapped.contains("citation gap"), "{gapped}");
}

#[test]
fn failed_wiki_build_preserves_the_existing_output_tree() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("repo").to_string_lossy().to_string();
    let out = dir.path().join("site");
    seed_full_state(dir.path(), &repo);
    std::fs::create_dir_all(out.join("requirements")).unwrap();
    std::fs::write(out.join("requirements/req_sah"), "blocking file").unwrap();
    std::fs::write(out.join("index.html"), "existing index").unwrap();

    Command::cargo_bin("provenance")
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
        .failure()
        .stderr(predicates::str::contains("unowned output path"));

    assert_eq!(
        std::fs::read_to_string(out.join("index.html")).unwrap(),
        "existing index"
    );
    assert_eq!(
        std::fs::read_to_string(out.join("requirements/req_sah")).unwrap(),
        "blocking file"
    );
    assert!(!out.join("requirements/req_gap/index.html").exists());
    assert!(!out.join("assets/provenance-wiki.css").exists());
}

#[test]
fn rebuild_removes_only_files_declared_by_generator_ownership() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("repo").to_string_lossy().to_string();
    let out = dir.path().join("site");
    seed_full_state(dir.path(), &repo);

    Command::cargo_bin("provenance")
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
        .success();

    let manifest_path = out.join(".provenance-wiki-output.json");
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&manifest_path).unwrap()).unwrap();
    manifest["files"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::Value::String("obsolete.html".to_string()));
    std::fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
    std::fs::write(out.join("obsolete.html"), "owned stale page").unwrap();
    std::fs::write(out.join("customer.txt"), "caller owned").unwrap();

    Command::cargo_bin("provenance")
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
        .success();

    assert!(!out.join("obsolete.html").exists());
    assert_eq!(
        std::fs::read_to_string(out.join("customer.txt")).unwrap(),
        "caller owned"
    );
}
