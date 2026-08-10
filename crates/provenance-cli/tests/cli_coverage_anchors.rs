use assert_cmd::Command;
use serde_json::Value;

struct Fixture {
    _temp: tempfile::TempDir,
    repo: std::path::PathBuf,
    source: std::path::PathBuf,
    baseline: std::path::PathBuf,
}

impl Fixture {
    fn new(source: &str) -> Self {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().to_path_buf();
        let source_dir = repo.join("src");
        let source_path = source_dir.join("rules.rs");
        std::fs::create_dir_all(&source_dir).unwrap();
        std::fs::write(&source_path, source).unwrap();

        provenance(&[
            "init",
            "--path",
            repo.to_str().unwrap(),
            "--scope",
            "default",
            "--path-prefix",
            ".",
        ]);
        provenance(&[
            "rules",
            "create",
            "--repo",
            repo.to_str().unwrap(),
            "--scope",
            "default",
            "--id",
            "rule_anchor",
            "--statement",
            "Anchor the deciding function",
            "--severity",
            "high",
        ]);

        let fixture = Self {
            baseline: repo.join("baseline.json"),
            source: source_path,
            repo,
            _temp: temp,
        };
        let baseline = fixture.scan(None, false);
        std::fs::write(
            &fixture.baseline,
            serde_json::to_vec_pretty(&baseline).unwrap(),
        )
        .unwrap();
        fixture
    }

    fn scan(&self, baseline: Option<&std::path::Path>, validate_rules: bool) -> Value {
        self.scan_at(self.source.parent().unwrap(), baseline, validate_rules)
    }

    fn scan_at(
        &self,
        path: &std::path::Path,
        baseline: Option<&std::path::Path>,
        validate_rules: bool,
    ) -> Value {
        let mut command = Command::cargo_bin("provenance").unwrap();
        command.args([
            "coverage",
            "scan",
            "--repo",
            self.repo.to_str().unwrap(),
            "--path",
            path.to_str().unwrap(),
            "--scope",
            "default",
            "--format",
            "json",
        ]);
        if let Some(baseline) = baseline {
            command.args(["--baseline", baseline.to_str().unwrap()]);
        }
        if validate_rules {
            command.arg("--validate-rules");
        }
        let output = command.assert().success().get_output().stdout.clone();
        serde_json::from_slice(&output).unwrap()
    }

    fn rescan(&self) -> Value {
        self.scan(Some(&self.baseline), true)
    }
}

fn provenance(args: &[&str]) {
    Command::cargo_bin("provenance")
        .unwrap()
        .args(args)
        .assert()
        .success();
}

fn binding<'a>(report: &'a Value, item: &str, state: &str) -> &'a Value {
    report["bindings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|binding| {
            binding["item_name"] == item && binding["anchor_state"].as_str() == Some(state)
        })
        .unwrap_or_else(|| panic!("missing {state} anchor for {item}: {report:#}"))
}

fn annotation<'a>(report: &'a Value, function: &str, state: &str) -> &'a Value {
    report["annotations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|annotation| {
            annotation["function_name"] == function
                && annotation["anchor_state"].as_str() == Some(state)
        })
        .unwrap_or_else(|| panic!("missing {state} anchor for {function}: {report:#}"))
}

const ORIGINAL: &str = "#[rule(\"rule_anchor\")]\n\
fn decide_anchor() {}\n\n\
#[verifies(\"rule_anchor\", examples)]\n\
fn verifies_anchor() {}\n";

#[test]
fn moved_binding_relocates_by_symbol_and_hash() {
    let fixture = Fixture::new(ORIGINAL);
    std::fs::write(
        &fixture.source,
        "#[rule(\"rule_anchor\")]\nfn decide_anchor() {}\n\nfn helper() {}\n\n\
         #[verifies(\"rule_anchor\", examples)]\nfn verifies_anchor() {}\n",
    )
    .unwrap();

    let report = fixture.rescan();
    let moved = binding(&report, "verifies_anchor", "moved");

    assert_eq!(moved["original_line"], 4);
    assert_eq!(moved["line"], 6);
    assert!(moved["anchor"]["content_hash"]
        .as_str()
        .unwrap()
        .starts_with("sha256:"));
    assert!(!report["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|warning| warning["message"].as_str().unwrap().contains("gone")));
}

#[test]
fn edited_binding_line_is_gone_not_relocated() {
    let fixture = Fixture::new(ORIGINAL);
    std::fs::write(
        &fixture.source,
        ORIGINAL.replace("rule_anchor\", examples", "rule_anchor\", exhaustion"),
    )
    .unwrap();

    let report = fixture.rescan();
    let gone = binding(&report, "verifies_anchor", "gone");

    assert_eq!(gone["line"], 4);
    assert!(gone.get("original_line").is_none());
    assert!(report["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|warning| warning["message"].as_str().unwrap().contains("gone")));
}

#[test]
fn untouched_binding_anchor_stays_unchanged() {
    let fixture = Fixture::new(ORIGINAL);

    let report = fixture.rescan();
    let unchanged = binding(&report, "verifies_anchor", "unchanged");

    assert_eq!(unchanged["line"], 4);
    assert!(unchanged.get("original_line").is_none());
    assert_eq!(unchanged["anchor"]["symbol"], "verifies_anchor");
}

#[test]
fn moved_annotation_relocates_with_its_function() {
    let fixture = Fixture::new(
        "// @provenance rule: rule_anchor\n// @provenance verification: examples\n\
         fn verifies_anchor() {}\n",
    );
    std::fs::write(
        &fixture.source,
        "fn helper() {}\n\n// @provenance rule: rule_anchor\n\
         // @provenance verification: examples\nfn verifies_anchor() {}\n",
    )
    .unwrap();

    let report = fixture.rescan();
    let moved = annotation(&report, "verifies_anchor", "moved");

    assert_eq!(moved["original_line"], 1);
    assert_eq!(moved["line"], 3);
}

#[test]
fn baseline_sites_outside_the_current_scan_are_not_called_gone() {
    let fixture = Fixture::new(ORIGINAL);
    std::fs::write(
        fixture.source.parent().unwrap().join("other.rs"),
        "#[verifies(\"rule_anchor\", examples)]\nfn outside_current_scan() {}\n",
    )
    .unwrap();
    let baseline = fixture.scan(None, false);
    std::fs::write(
        &fixture.baseline,
        serde_json::to_vec_pretty(&baseline).unwrap(),
    )
    .unwrap();

    let report = fixture.scan_at(&fixture.source, Some(&fixture.baseline), true);

    assert!(!report["bindings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|site| site["item_name"] == "outside_current_scan"));
    assert!(!report["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|warning| warning["message"].as_str().unwrap().contains("gone")));
}

#[test]
fn deleted_file_sites_are_gone_when_the_parent_is_scanned() {
    let fixture = Fixture::new(ORIGINAL);
    std::fs::remove_file(&fixture.source).unwrap();

    let report = fixture.rescan();

    binding(&report, "verifies_anchor", "gone");
    assert!(report["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|warning| warning["message"].as_str().unwrap().contains("gone")));
}

#[test]
fn relocation_ignores_absolute_vs_relative_path_spelling() {
    let fixture = Fixture::new(ORIGINAL);
    std::fs::write(
        &fixture.source,
        "#[rule(\"rule_anchor\")]\nfn decide_anchor() {}\n\nfn helper() {}\n\n\
         #[verifies(\"rule_anchor\", examples)]\nfn verifies_anchor() {}\n",
    )
    .unwrap();
    let output = Command::cargo_bin("provenance")
        .unwrap()
        .current_dir(&fixture.repo)
        .args([
            "coverage",
            "scan",
            "--repo",
            ".",
            "--path",
            "src/../src",
            "--scope",
            "default",
            "--format",
            "json",
            "--baseline",
            fixture.baseline.to_str().unwrap(),
            "--validate-rules",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let report: Value = serde_json::from_slice(&output).unwrap();

    let moved = binding(&report, "verifies_anchor", "moved");
    assert_eq!(moved["line"], 6);
}

#[test]
fn duplicate_symbol_and_hash_are_not_guessed_as_moved_or_gone() {
    let fixture = Fixture::new(
        "mod first {\n#[verifies(\"rule_anchor\", examples)]\nfn duplicate() {}\n}\n\n\
         mod second {\n#[verifies(\"rule_anchor\", examples)]\nfn duplicate() {}\n}\n",
    );
    std::fs::write(
        &fixture.source,
        "mod second {\n#[verifies(\"rule_anchor\", examples)]\nfn duplicate() {}\n}\n",
    )
    .unwrap();

    let report = fixture.rescan();
    let duplicate_sites = report["bindings"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|site| site["item_name"] == "duplicate")
        .collect::<Vec<_>>();

    assert_eq!(duplicate_sites.len(), 1);
    assert_eq!(duplicate_sites[0]["anchor_state"], "unchanged");
    assert!(!report["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|warning| warning["message"].as_str().unwrap().contains("gone")));
}
