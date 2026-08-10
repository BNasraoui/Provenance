use super::current_git_commit;
use camino::Utf8PathBuf;

#[test]
fn coverage_commit_is_read_from_the_selected_repository() {
    let dir = tempfile::tempdir().unwrap();
    let repo = Utf8PathBuf::from_path_buf(dir.path().join("repo")).unwrap();
    run_git(["init", repo.as_str()]);
    std::fs::write(repo.join("tracked.txt"), "tracked\n").unwrap();
    run_git(["-C", repo.as_str(), "add", "tracked.txt"]);
    run_git([
        "-C",
        repo.as_str(),
        "-c",
        "user.name=Provenance Test",
        "-c",
        "user.email=test@example.invalid",
        "commit",
        "-m",
        "fixture",
    ]);

    let expected = git_stdout(["-C", repo.as_str(), "rev-parse", "--short", "HEAD"]);

    assert_eq!(current_git_commit(&repo).unwrap(), expected);
}

fn run_git<const N: usize>(args: [&str; N]) {
    let output = std::process::Command::new("git")
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn git_stdout<const N: usize>(args: [&str; N]) -> String {
    let output = std::process::Command::new("git")
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}
