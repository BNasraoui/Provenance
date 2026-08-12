//! A distributed binary has to be able to say which build it is.

use assert_cmd::Command;

/// The version string names the binary as invoked, not the crate, and carries
/// the workspace version every crate shares.
#[test]
fn version_flag_reports_the_binary_name_and_workspace_version() {
    let expected = format!("provenance {}\n", env!("CARGO_PKG_VERSION"));

    for flag in ["--version", "-V"] {
        Command::cargo_bin("provenance")
            .unwrap()
            .arg(flag)
            .assert()
            .success()
            .stdout(expected.clone());
    }
}
