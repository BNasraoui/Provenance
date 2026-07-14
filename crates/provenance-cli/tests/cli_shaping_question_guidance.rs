#[path = "shaping_support/provenance.rs"]
mod provenance;

use predicates::str::contains;
use provenance::provenance;

#[test]
fn cli_questions_create_help_includes_sizing_guidance() {
    provenance(&["questions", "create", "--help"])
        .success()
        .stdout(contains(
            "A question should be resolvable in one agent session",
        ))
        .stdout(contains("otherwise it is fog or needs decomposition"));
}
