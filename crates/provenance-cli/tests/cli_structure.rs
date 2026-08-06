use std::{fs, path::PathBuf, process::Command};

const RUST_FILE_LINE_LIMIT: usize = 500;

#[test]
fn tracked_rust_files_stay_below_the_hard_line_limit() {
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("CLI crate is nested under the workspace root")
        .to_path_buf();
    let output = Command::new("git")
        .args([
            "-C",
            workspace.to_str().expect("workspace path is UTF-8"),
            "ls-files",
            "-z",
            "--",
            ":(glob)**/*.rs",
        ])
        .output()
        .expect("run git ls-files for the Rust line audit");
    assert!(output.status.success(), "git ls-files failed");
    let files = output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|path| !path.is_empty())
        .map(|path| String::from_utf8(path.to_vec()).expect("tracked Rust path is UTF-8"))
        .collect::<Vec<_>>();
    assert!(!files.is_empty(), "tracked Rust line audit found no files");

    for relative_path in files {
        let path = workspace.join(&relative_path);
        let source = fs::read_to_string(&path).expect("read tracked Rust file");
        let line_count = source.lines().count();
        assert!(
            line_count < RUST_FILE_LINE_LIMIT,
            "{relative_path} has {line_count} lines; tracked Rust files must stay below {RUST_FILE_LINE_LIMIT}",
        );
    }
}
