use assert_cmd::Command;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::process::Command as StdCommand;
use std::time::{Duration, Instant};

#[test]
fn wiki_build_writes_static_pages_and_stylesheet() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("repo");
    let repo = repo.to_string_lossy().to_string();
    let out = dir.path().join("site");
    seed_state(dir.path(), &repo);

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
fn wiki_serve_serves_pages_stylesheet_and_not_found() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("repo");
    let repo = repo.to_string_lossy().to_string();
    seed_state(dir.path(), &repo);

    let port = free_port();
    let mut child = StdCommand::new(assert_cmd::cargo::cargo_bin("provenance"))
        .args([
            "wiki",
            "serve",
            "--repo",
            &repo,
            "--host",
            "127.0.0.1",
            "--port",
            &port.to_string(),
        ])
        .spawn()
        .unwrap();

    let index = wait_for_http(port, "/");
    let stylesheet = wait_for_http(port, "/assets/provenance-wiki.css");
    let requirement = wait_for_http(port, "/requirements/req_sah/");
    let bare_route = wait_for_http(port, "/requirements/req_sah");
    let missing = wait_for_http(port, "/nope/");
    child.kill().ok();
    child.wait().ok();

    assert!(index.contains("200 OK"), "{index}");
    assert!(index.contains("- Provenance Wiki</title>"), "{index}");
    assert!(index.contains("href=\"/requirements/req_sah/\""), "{index}");

    assert!(stylesheet.contains("200 OK"), "{stylesheet}");
    assert!(stylesheet.contains("text/css"), "{stylesheet}");
    assert!(stylesheet.contains("--pv-"), "{stylesheet}");

    assert!(requirement.contains("200 OK"), "{requirement}");
    assert!(
        requirement.contains("Support at Home shall be traceable"),
        "{requirement}"
    );

    assert!(bare_route.contains("200 OK"), "{bare_route}");
    assert!(
        bare_route.contains("Support at Home shall be traceable"),
        "{bare_route}"
    );

    assert!(missing.contains("404 Not Found"), "{missing}");
    assert!(missing.contains("Page not found"), "{missing}");
}

#[allow(clippy::too_many_lines)]
fn seed_state(dir: &std::path::Path, repo: &str) {
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "init",
            "--path",
            repo,
            "--scope",
            "default",
            "--path-prefix",
            ".",
        ])
        .assert()
        .success();

    let import_path = dir.join("state.json");
    std::fs::write(
        &import_path,
        r#"{
  "scope": "default",
  "sources": [{
    "schema_version": 1,
    "scope_id": "default",
    "id": "source_sah",
    "name": "Support at Home",
    "source_type": "legislation",
    "url": "https://example.test/sah",
    "reference": "Department guidance"
  }],
  "requirements": [{
    "schema_version": 1,
    "scope_id": "default",
    "id": "req_gap",
    "statement": "Uncited requirement",
    "status": "active",
    "source_refs": []
  }, {
    "schema_version": 1,
    "scope_id": "default",
    "id": "req_sah",
    "statement": "Support at Home shall be traceable",
    "status": "active",
    "source_refs": [{"source_id": "source_sah", "clause": "Program overview"}]
  }],
  "resolutions": [{
    "schema_version": 1,
    "scope_id": "default",
    "id": "res_sah",
    "title": "SAH extraction",
    "position": "Keep as draft extraction",
    "rationale": "Needs human review",
    "status": "approved",
    "review_on": null,
    "review_triggers": []
  }],
  "rules": [{
    "schema_version": 1,
    "scope_id": "default",
    "id": "rule_sah_001",
    "rule_code": "SAH-001",
    "name": "SAH rule",
    "statement": "Draft rule shall stay draft",
    "status": "active",
    "severity": "high",
    "rule_type": "business",
    "modality": "obligation",
    "source_document": "Example-API-main/src/example.php",
    "source_section": "lines 1-3",
    "expression": {},
    "inputs": []
  }],
  "edges": [],
  "threads": [],
  "messages": []
}"#,
    )
    .unwrap();

    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "import",
            "--repo",
            repo,
            "--scope",
            "default",
            "--input",
            &import_path.to_string_lossy(),
            "--format",
            "json",
        ])
        .assert()
        .success();

    create_edge(
        repo,
        "resolves",
        "resolution",
        "res_sah",
        "requirement",
        "req_sah",
    );
    create_edge(
        repo,
        "produces",
        "resolution",
        "res_sah",
        "rule",
        "rule_sah_001",
    );
}

fn create_edge(
    repo: &str,
    edge_type: &str,
    from_type: &str,
    from_id: &str,
    to_type: &str,
    to_id: &str,
) {
    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "edges",
            "create",
            "--repo",
            repo,
            "--scope",
            "default",
            "--type",
            edge_type,
            "--from-type",
            from_type,
            "--from-id",
            from_id,
            "--to-type",
            to_type,
            "--to-id",
            to_id,
        ])
        .assert()
        .success();
}

fn free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}

fn wait_for_http(port: u16, path: &str) -> String {
    let deadline = Instant::now() + Duration::from_secs(10);
    let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
    let mut last_error = None;

    while Instant::now() < deadline {
        // Treat connect, write, and read as a single attempt: a listener
        // that has only just bound (or is still behind the spawning
        // process's startup) can accept the TCP handshake and then reset
        // the connection before it is actually ready to serve a full
        // request/response cycle. Retrying the whole attempt on any IO
        // error here (not just on connection refused) is what makes this
        // robust against that startup race instead of panicking on it.
        match attempt_http_request(addr, path) {
            Ok(response) => return response,
            Err(error) => {
                last_error = Some(error);
                std::thread::sleep(Duration::from_millis(50));
            }
        }
    }

    panic!("server did not respond: {last_error:?}");
}

fn attempt_http_request(addr: SocketAddr, path: &str) -> std::io::Result<String> {
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_millis(150))?;
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n"
    )?;
    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    Ok(response)
}
