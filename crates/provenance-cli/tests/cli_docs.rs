use assert_cmd::Command;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::process::Command as StdCommand;
use std::time::{Duration, Instant};

#[test]
fn docs_check_accepts_plain_markdown_tree_and_reports_inferred_pages() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path();
    std::fs::create_dir_all(repo.join("docs/guide")).unwrap();
    std::fs::write(
        repo.join("docs/index.md"),
        "# Agent Handbook\n\nSee [Install](guide/install.md).\n",
    )
    .unwrap();
    std::fs::write(
        repo.join("docs/guide/install.md"),
        "# Installation\n\nBack to [home](../index.md).\n",
    )
    .unwrap();

    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "docs",
            "check",
            "--repo",
            &repo.to_string_lossy(),
            "--format",
            "json",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains(r#""status": "ok""#))
        .stdout(predicates::str::contains(r#""homepage_route": "/""#))
        .stdout(predicates::str::contains(r#""route": "/guide/install/""#))
        .stdout(predicates::str::contains(r#""title": "Installation""#))
        .stdout(predicates::str::contains(r#""page_count": 2"#));
}

#[test]
fn docs_check_fails_on_broken_relative_markdown_links() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path();
    std::fs::create_dir_all(repo.join("docs")).unwrap();
    std::fs::write(
        repo.join("docs/index.md"),
        "# Agent Handbook\n\nSee [Missing](missing.md).\n",
    )
    .unwrap();

    Command::cargo_bin("provenance")
        .unwrap()
        .args(["docs", "check", "--repo", &repo.to_string_lossy()])
        .assert()
        .failure()
        .stderr(predicates::str::contains("broken docs link"))
        .stderr(predicates::str::contains("docs/index.md"))
        .stderr(predicates::str::contains("missing.md"));
}

#[test]
fn docs_check_uses_readme_as_homepage_when_docs_index_is_absent() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path();
    std::fs::create_dir_all(repo.join("docs/guide")).unwrap();
    std::fs::write(
        repo.join("README.md"),
        "# Project Handbook\n\nContinue to [Install](docs/guide/install.md).\n",
    )
    .unwrap();
    std::fs::write(
        repo.join("docs/guide/install.md"),
        "# Installation\n\nBack to [home](../../README.md).\n",
    )
    .unwrap();

    Command::cargo_bin("provenance")
        .unwrap()
        .args([
            "docs",
            "check",
            "--repo",
            &repo.to_string_lossy(),
            "--format",
            "json",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains(r#""homepage_route": "/""#))
        .stdout(predicates::str::contains(r#""source_path": "README.md""#))
        .stdout(predicates::str::contains(r#""title": "Project Handbook""#))
        .stdout(predicates::str::contains(r#""route": "/guide/install/""#))
        .stdout(predicates::str::contains(r#""page_count": 2"#));
}

#[test]
fn docs_serve_renders_semantic_html_and_rewrites_markdown_links() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path();
    std::fs::create_dir_all(repo.join("docs/guide")).unwrap();
    std::fs::write(
        repo.join("docs/index.md"),
        "# Agent Handbook\n\nSee [Install](guide/install.md).\n",
    )
    .unwrap();
    std::fs::write(repo.join("docs/guide/install.md"), "# Installation\n").unwrap();

    let port = free_port();
    let mut child = StdCommand::new(assert_cmd::cargo::cargo_bin("provenance"))
        .args([
            "docs",
            "serve",
            "--repo",
            &repo.to_string_lossy(),
            "--host",
            "127.0.0.1",
            "--port",
            &port.to_string(),
        ])
        .spawn()
        .unwrap();

    let response = wait_for_http(port, "/");
    child.kill().ok();
    child.wait().ok();

    assert!(response.contains("200 OK"), "{response}");
    assert!(response.contains("<title>Agent Handbook - Provenance Docs</title>"));
    assert!(response.contains("<nav aria-label=\"Docs navigation\">"));
    assert!(response.contains("<main>"));
    assert!(response.contains("<article>"));
    assert!(response.contains("href=\"/guide/install/\""));
    assert!(!response.contains("href=\"guide/install.md\""));
    assert!(!response.contains("class="));
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
