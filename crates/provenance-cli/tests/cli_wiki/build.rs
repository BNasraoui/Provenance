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
    assert!(
        !out.join("assets/search-index.json").exists(),
        "the rendered DOM is the only search publication"
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
fn wiki_build_reports_per_page_write_failures_without_aborting_the_rest() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("repo").to_string_lossy().to_string();
    let out = dir.path().join("site");
    seed_full_state(dir.path(), &repo);
    std::fs::create_dir_all(out.join("requirements")).unwrap();
    std::fs::write(out.join("requirements/req_sah"), "blocking file").unwrap();

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
        .stderr(predicates::str::contains("req_sah"));

    for (path, text) in [
        ("requirements/req_gap/index.html", "citation gap"),
        ("index.html", "Provenance Wiki"),
        ("resolutions/res_sah/index.html", "SAH extraction"),
        ("rules/rule_sah_001/index.html", "SAH rule"),
        ("sources/source_sah/index.html", "Support at Home"),
    ] {
        let page = std::fs::read_to_string(out.join(path)).unwrap();
        assert!(page.contains(text), "{path}: {page}");
    }
}
