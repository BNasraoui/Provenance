use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;

fn provenance() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("provenance"))
}

#[test]
fn top_level_help_keeps_commands_from_each_cli_domain() {
    provenance().arg("--help").assert().success().stdout(
        contains("requirements")
            .and(contains("questions"))
            .and(contains("proposals"))
            .and(contains("docs")),
    );
}

#[test]
fn nested_help_parses_commands_from_each_cli_domain() {
    for command in [
        &["requirements", "--help"][..],
        &["questions", "--help"][..],
        &["proposals", "--help"][..],
        &["docs", "--help"][..],
    ] {
        provenance().args(command).assert().success();
    }
}
