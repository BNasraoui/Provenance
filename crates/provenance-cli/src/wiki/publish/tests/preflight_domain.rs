//! Exhaustion over the state `preflight` probes before a publication starts.
//!
//! The decision has a finite domain once the probed state is written out: the
//! output path is absent, a real directory, a symlink, or a regular file, and
//! each of the five transaction artifacts is either present or absent. That is
//! 4 x 2^5 = 128 states, and every one of them is built here as real files in
//! a temporary directory and run through `preflight`.
//!
//! The expected answer comes from an oracle restated from the rule, not from
//! the code under test, and every case also checks that nothing on disk moved:
//! a refusal reports residue, it does not adopt or delete it. Each refusal over
//! residue is checked to name the whole of it, so the operator's next move is
//! informed by everything the run found rather than by the first leftover.

use super::{artifact, utf8, PublicationOutput};
use crate::wiki::publish::{preflight, PublishError, TransactionDirectory};
use camino::Utf8Path;
use provenance_macros::verifies;
use std::os::unix::fs::symlink;
use std::path::Path;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum OutputKind {
    Absent,
    Directory,
    Symlink,
    RegularFile,
}

const OUTPUT_KINDS: [OutputKind; 4] = [
    OutputKind::Absent,
    OutputKind::Directory,
    OutputKind::Symlink,
    OutputKind::RegularFile,
];

/// The artifact roles an interrupted publication can leave behind, named here
/// independently of the production code that mints them.
const ARTIFACT_ROLES: [&str; 5] = ["lock", "lock.cleanup", "stage", "stage.cleanup", "backup"];

/// Independent restatement of the preflight decision, written from the rule:
/// a publication may start only when the output path holds either nothing or a
/// real directory, and no artifact of an earlier run sits beside it.
fn should_accept(output: OutputKind, residue: &[&str]) -> bool {
    matches!(output, OutputKind::Absent | OutputKind::Directory) && residue.is_empty()
}

fn create_output(temp: &Path, output: &Utf8Path, kind: OutputKind) {
    match kind {
        OutputKind::Absent => {}
        OutputKind::Directory => std::fs::create_dir(output).unwrap(),
        OutputKind::Symlink => {
            let target = utf8(temp.join("symlink-target"));
            std::fs::create_dir(&target).unwrap();
            std::fs::write(target.join("caller.txt"), "keep me").unwrap();
            symlink(&target, output).unwrap();
        }
        OutputKind::RegularFile => std::fs::write(output, "caller bytes").unwrap(),
    }
}

/// Writes crash residue for one role, in the shape a real interrupted run
/// leaves: the two lock roles are files, the three tree roles are directories.
fn create_residue(output: &Utf8Path, role: &str) {
    let path = artifact(output, role);
    if role.starts_with("lock") {
        std::fs::write(path, "residue").unwrap();
    } else {
        std::fs::create_dir(&path).unwrap();
        std::fs::write(path.join("caller"), "residue").unwrap();
    }
}

fn assert_output_untouched(output: &Utf8Path, kind: OutputKind) {
    match kind {
        OutputKind::Absent => assert!(
            output.symlink_metadata().is_err(),
            "preflight created the output at {output}"
        ),
        OutputKind::Directory => {
            let metadata = output.symlink_metadata().unwrap();
            assert!(metadata.is_dir(), "preflight replaced the output directory");
            assert!(
                std::fs::read_dir(output).unwrap().next().is_none(),
                "preflight wrote into the output directory"
            );
        }
        OutputKind::Symlink => assert!(
            output.symlink_metadata().unwrap().file_type().is_symlink(),
            "preflight resolved or replaced the output symlink"
        ),
        OutputKind::RegularFile => assert_eq!(
            std::fs::read_to_string(output).unwrap(),
            "caller bytes",
            "preflight disturbed a caller file at the output path"
        ),
    }
}

/// A refusal over residue has to name every artifact that is there, in one
/// error: an operator who clears what the message lists must not be sent back
/// for a leftover it kept quiet about.
fn assert_every_artifact_is_named(output: &Utf8Path, residue: &[&str], error: PublishError) {
    let expected: Vec<_> = residue.iter().map(|role| artifact(output, role)).collect();
    match error {
        PublishError::AmbiguousArtifacts { paths, unsafe_lock } => {
            assert_eq!(
                paths, expected,
                "preflight named {paths:?} of the residue {residue:?}"
            );
            assert_eq!(
                unsafe_lock, None,
                "preflight called a regular-file lock unsafe"
            );
        }
        other => panic!("preflight refused the residue {residue:?} with {other}"),
    }
}

fn assert_residue_untouched(output: &Utf8Path, residue: &[&str]) {
    for role in residue {
        let path = artifact(output, role);
        let expected = if role.starts_with("lock") {
            path.clone()
        } else {
            path.join("caller")
        };
        assert_eq!(
            std::fs::read_to_string(&expected).unwrap(),
            "residue",
            "preflight disturbed the {role} artifact"
        );
    }
}

/// Runs every (output kind, residue subset) state and checks the verdict
/// against the oracle. Refusals must also leave the directory exactly as found.
#[test]
#[verifies("rule_publish_preflight", exhaustion)]
fn accepts_exactly_a_clean_directory_output_with_no_residue() {
    for kind in OUTPUT_KINDS {
        for mask in 0..(1u32 << ARTIFACT_ROLES.len()) {
            let residue: Vec<&str> = ARTIFACT_ROLES
                .iter()
                .enumerate()
                .filter(|(index, _)| mask & (1u32 << index) != 0)
                .map(|(_, role)| *role)
                .collect();
            let temp = tempfile::tempdir().unwrap();
            let output = utf8(temp.path().join("wiki"));
            create_output(temp.path(), &output, kind);
            for role in &residue {
                create_residue(&output, role);
            }

            let transaction = TransactionDirectory::open(&output).unwrap();
            let decision = preflight(&PublicationOutput::custom(output.clone()), &transaction);
            let accepted = decision.is_ok();

            assert_eq!(
                accepted,
                should_accept(kind, &residue),
                "preflight {} a {kind:?} output carrying residue {residue:?}",
                if accepted { "accepted" } else { "refused" }
            );
            if !residue.is_empty() && matches!(kind, OutputKind::Absent | OutputKind::Directory) {
                assert_every_artifact_is_named(
                    &output,
                    &residue,
                    decision.err().expect("a refusal over residue"),
                );
            }
            assert_output_untouched(&output, kind);
            assert_residue_untouched(&output, &residue);
        }
    }
}
