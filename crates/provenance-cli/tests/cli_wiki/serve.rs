use super::support::seed_full_state;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::process::Command;
use std::time::{Duration, Instant};

#[test]
fn wiki_serve_serves_pages_stylesheet_and_not_found() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("repo").to_string_lossy().to_string();
    seed_full_state(dir.path(), &repo);
    let port = free_port();
    let mut child = Command::new(assert_cmd::cargo::cargo_bin("provenance"))
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
    let search_index = wait_for_http(port, "/assets/search-index.json");
    let requirement = wait_for_http(port, "/requirements/req_sah/");
    let bare_route = wait_for_http(port, "/requirements/req_sah");
    let missing = wait_for_http(port, "/nope/");
    child.kill().ok();
    child.wait().ok();

    assert!(index.contains("200 OK"), "{index}");
    assert!(index.contains("href=\"/requirements/req_sah/\""), "{index}");
    assert!(stylesheet.contains("200 OK"), "{stylesheet}");
    assert!(stylesheet.contains("text/css"), "{stylesheet}");
    assert!(search_index.contains("404 Not Found"), "{search_index}");
    assert!(requirement.contains("200 OK"), "{requirement}");
    assert!(
        requirement.contains("Support at Home shall be traceable"),
        "{requirement}"
    );
    assert!(bare_route.contains("200 OK"), "{bare_route}");
    assert!(missing.contains("404 Not Found"), "{missing}");
    assert!(missing.contains("Page not found"), "{missing}");
}

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn wait_for_http(port: u16, path: &str) -> String {
    let deadline = Instant::now() + Duration::from_secs(10);
    let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
    let mut last_error = None;
    while Instant::now() < deadline {
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
