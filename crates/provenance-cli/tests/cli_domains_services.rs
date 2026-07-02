use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn cli_domains_services_and_bindings_roundtrip_materialize_and_export() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("repo").to_string_lossy().to_string();
    let import_repo = dir.path().join("imported").to_string_lossy().to_string();
    let export_path = dir.path().join("export.json").to_string_lossy().to_string();
    let import_export_path = dir
        .path()
        .join("import-export.json")
        .to_string_lossy()
        .to_string();

    init(&repo);
    create_domain(&repo);
    create_requirement(&repo);
    create_rule(&repo);
    create_service(&repo);
    create_service_binding(&repo);
    verify_materialized_lists(&repo);
    export_scope(&repo, &export_path);
    assert_export_contains_xep_records(&export_path);
    import_export_roundtrip(&import_repo, &export_path, &import_export_path);
}

fn provenance(args: &[&str]) -> assert_cmd::assert::Assert {
    Command::cargo_bin("provenance")
        .unwrap()
        .args(args)
        .assert()
}

fn init(repo: &str) {
    provenance(&[
        "init",
        "--path",
        repo,
        "--scope",
        "default",
        "--path-prefix",
        ".",
    ])
    .success();
}

fn create_domain(repo: &str) {
    provenance(&[
        "domains",
        "create",
        "--repo",
        repo,
        "--scope",
        "default",
        "--id",
        "domain_payroll",
        "--name",
        "Payroll",
        "--description",
        "Payroll compliance",
        "--color",
        "#3b82f6",
        "--format",
        "json",
    ])
    .success()
    .stdout(contains("domain_payroll"))
    .stdout(contains(r##""color": "#3b82f6""##));
}

fn create_requirement(repo: &str) {
    provenance(&[
        "requirements",
        "create",
        "--repo",
        repo,
        "--scope",
        "default",
        "--id",
        "req_overtime",
        "--statement",
        "Overtime must be traceable",
        "--status",
        "discovery",
        "--domain-id",
        "domain_payroll",
        "--format",
        "json",
    ])
    .success()
    .stdout(contains(r#""domain_id": "domain_payroll""#));
}

fn create_rule(repo: &str) {
    provenance(&[
        "rules",
        "create",
        "--repo",
        repo,
        "--scope",
        "default",
        "--id",
        "rule_overtime",
        "--rule-code",
        "PAY-001",
        "--requirement-id",
        "req_overtime",
        "--statement",
        "Pay overtime after threshold",
        "--status",
        "active",
        "--severity",
        "high",
        "--format",
        "json",
    ])
    .success();
}

fn create_service(repo: &str) {
    provenance(&[
        "services",
        "create",
        "--repo",
        repo,
        "--scope",
        "default",
        "--id",
        "service_payroll_api",
        "--name",
        "payroll-api",
        "--description",
        "Calculates payroll",
        "--owner",
        "platform",
        "--repository",
        "github.com/example/payroll-api",
        "--environment",
        "production",
        "--tier",
        "critical",
        "--external-id",
        "backstage:component/payroll-api",
        "--status",
        "active",
        "--format",
        "json",
    ])
    .success()
    .stdout(contains("service_payroll_api"))
    .stdout(contains(r#""environment": "production""#));
}

fn create_service_binding(repo: &str) {
    provenance(&[
        "service-bindings",
        "create",
        "--repo",
        repo,
        "--scope",
        "default",
        "--rule-id",
        "rule_overtime",
        "--service-id",
        "service_payroll_api",
        "--binding-type",
        "enforces",
        "--format",
        "json",
    ])
    .success()
    .stdout(contains(
        "service_binding_rule_overtime_service_payroll_api_enforces",
    ));
}

fn verify_materialized_lists(repo: &str) {
    provenance(&["materialize", "--repo", repo, "--format", "json"])
        .success()
        .stdout(contains(r#""records_loaded""#));
    provenance(&[
        "domains", "list", "--repo", repo, "--scope", "default", "--format", "json",
    ])
    .success()
    .stdout(contains("domain_payroll"));
    provenance(&[
        "services", "list", "--repo", repo, "--scope", "default", "--format", "json",
    ])
    .success()
    .stdout(contains("service_payroll_api"));
    provenance(&[
        "service-bindings",
        "list",
        "--repo",
        repo,
        "--scope",
        "default",
        "--format",
        "json",
    ])
    .success()
    .stdout(contains("rule_overtime"));
}

fn export_scope(repo: &str, export_path: &str) {
    provenance(&[
        "export",
        "--repo",
        repo,
        "--scope",
        "default",
        "--format",
        "json",
        "--output",
        export_path,
    ])
    .success();
}

fn assert_export_contains_xep_records(export_path: &str) {
    let exported = std::fs::read_to_string(export_path).unwrap();
    assert!(exported.contains(r#""domains""#));
    assert!(exported.contains(r#""services""#));
    assert!(exported.contains(r#""service_bindings""#));
    assert!(exported.contains("domain_payroll"));
    assert!(exported.contains("service_payroll_api"));
}

fn import_export_roundtrip(import_repo: &str, export_path: &str, import_export_path: &str) {
    init(import_repo);
    provenance(&[
        "import",
        "--repo",
        import_repo,
        "--scope",
        "default",
        "--input",
        export_path,
        "--format",
        "json",
    ])
    .success();
    export_scope(import_repo, import_export_path);

    let imported_export = std::fs::read_to_string(import_export_path).unwrap();
    assert!(imported_export.contains("domain_payroll"));
    assert!(imported_export.contains("service_payroll_api"));
    assert!(imported_export.contains("service_binding_rule_overtime_service_payroll_api_enforces"));
}
