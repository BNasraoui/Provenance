use assert_cmd::Command;
use predicates::prelude::*;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

const GROUNDED_DESCRIPTION: &str = "Write specific, evidence-grounded statements for requirements, rules, sources, resolutions, and boundaries — not generic capability language. Use before calling `requirements create/update`, `rules create/update`, `sources create/update`, `resolutions create/update`, or `boundaries create`, especially for a root or mid-level requirement, a statement merging several candidates, or a resolution's position and rationale.";

const LEGACY_NAMES: [&str; 4] = [
    "shaping",
    "fork-tournament",
    "swarm-backtrace",
    "grounded-writing",
];

#[test]
fn cleanup_validates_payload_hash_and_header_placement() {
    let dir = tempfile::tempdir().unwrap();
    let roots = [".claude/skills", ".agents/skills"];
    for (index, root) in roots.iter().enumerate() {
        let valid = legacy_dir(dir.path(), root, LEGACY_NAMES[index]);
        write_managed_skill(&valid, "---\nname: old\n---\n", "payload\n");

        let forged = legacy_dir(dir.path(), root, LEGACY_NAMES[index + 2]);
        std::fs::create_dir_all(&forged).unwrap();
        std::fs::write(
            forged.join("SKILL.md"),
            "---\nname: old\n---\n<!-- Installed by provenance 0.1.0; content hash fnv1a64:0000000000000000 -->\npayload\n",
        )
        .unwrap();
    }

    install_local(dir.path()).success();

    for (index, root) in roots.iter().enumerate() {
        assert!(!legacy_dir(dir.path(), root, LEGACY_NAMES[index]).exists());
        assert!(legacy_dir(dir.path(), root, LEGACY_NAMES[index + 2]).exists());
    }
}

#[test]
fn cleanup_preserves_modified_payload_and_misplaced_valid_header() {
    let dir = tempfile::tempdir().unwrap();
    let modified = legacy_dir(dir.path(), ".claude/skills", "shaping");
    write_managed_skill(&modified, "---\nname: old\n---\n", "payload\n");
    std::fs::write(
        modified.join("SKILL.md"),
        format!(
            "---\nname: old\n---\n<!-- Installed by provenance 0.1.0; content hash fnv1a64:{} -->\nchanged\n",
            fnv1a64("payload\n")
        ),
    )
    .unwrap();
    let misplaced = legacy_dir(dir.path(), ".agents/skills", "fork-tournament");
    std::fs::create_dir_all(&misplaced).unwrap();
    std::fs::write(
        misplaced.join("SKILL.md"),
        format!(
            "<!-- Installed by provenance 0.1.0; content hash fnv1a64:{} -->\n---\nname: old\n---\npayload\n",
            fnv1a64("---\nname: old\n---\npayload\n")
        ),
    )
    .unwrap();

    install_local(dir.path()).success();

    assert!(modified.exists());
    assert!(misplaced.exists());
}

#[test]
fn cleanup_removes_only_skill_file_and_keeps_user_files() {
    let dir = tempfile::tempdir().unwrap();
    let legacy = legacy_dir(dir.path(), ".claude/skills", "swarm-backtrace");
    write_managed_skill(&legacy, "---\nname: old\n---\n", "payload\n");
    std::fs::write(legacy.join("notes.txt"), "mine\n").unwrap();

    install_local(dir.path()).success();

    assert!(!legacy.join("SKILL.md").exists());
    assert_eq!(
        std::fs::read_to_string(legacy.join("notes.txt")).unwrap(),
        "mine\n"
    );
}

#[test]
fn rerun_reports_legacy_cleanup_as_an_update() {
    let dir = tempfile::tempdir().unwrap();
    install_local(dir.path()).success();
    let legacy = legacy_dir(dir.path(), ".agents/skills", "shaping");
    write_managed_skill(&legacy, "---\nname: old\n---\n", "payload\n");

    let output = install_local(dir.path())
        .success()
        .get_output()
        .stdout
        .clone();
    let report: serde_json::Value = serde_json::from_slice(&output).unwrap();

    assert_eq!(report["status"], "updated");
    assert!(report["files"].as_array().unwrap().iter().any(|file| {
        file["path"].as_str() == Some(legacy.join("SKILL.md").to_str().unwrap())
            && file["status"] == "removed"
    }));
}

#[test]
fn global_cleanup_covers_both_skill_path_families() {
    let home = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    for root in [".claude/skills", ".agents/skills"] {
        let legacy = legacy_dir(home.path(), root, "grounded-writing");
        write_managed_skill(&legacy, "---\nname: old\n---\n", "payload\n");
    }

    Command::cargo_bin("provenance")
        .unwrap()
        .current_dir(cwd.path())
        .env("HOME", home.path())
        .args(["skills", "install", "--global", "--copy"])
        .assert()
        .success();

    for root in [".claude/skills", ".agents/skills"] {
        assert!(!legacy_dir(home.path(), root, "grounded-writing").exists());
    }
}

#[test]
fn later_claude_conflict_leaves_all_legacy_artifacts_untouched() {
    let dir = tempfile::tempdir().unwrap();
    let legacy = legacy_dir(dir.path(), ".agents/skills", "shaping");
    write_managed_skill(&legacy, "---\nname: old\n---\n", "payload\n");
    let agents = dir.path().join("AGENTS.md");
    let drifted =
        "before\n<!-- BEGIN PROVENANCE SKILLS -->\ndrift\n<!-- END PROVENANCE SKILLS -->\nafter\n";
    std::fs::write(&agents, drifted).unwrap();
    let conflict = dir
        .path()
        .join(".claude/skills/provenance-swarm-backtrace/SKILL.md");
    std::fs::create_dir_all(conflict.parent().unwrap()).unwrap();
    std::fs::write(&conflict, "foreign\n").unwrap();

    install_local(dir.path())
        .failure()
        .stderr(predicate::str::contains("exists and differs"));

    assert!(legacy.join("SKILL.md").exists());
    assert_eq!(std::fs::read_to_string(agents).unwrap(), drifted);
}

#[test]
fn canonical_conflict_leaves_all_legacy_artifacts_untouched() {
    let dir = tempfile::tempdir().unwrap();
    let legacy = legacy_dir(dir.path(), ".claude/skills", "fork-tournament");
    write_managed_skill(&legacy, "---\nname: old\n---\n", "payload\n");
    let conflict = dir
        .path()
        .join(".agents/skills/provenance-fork-tournament/SKILL.md");
    std::fs::create_dir_all(conflict.parent().unwrap()).unwrap();
    std::fs::write(&conflict, "foreign\n").unwrap();

    install_local(dir.path())
        .failure()
        .stderr(predicate::str::contains("exists and differs"));

    assert!(legacy.join("SKILL.md").exists());
}

#[test]
fn agents_marker_block_with_unverifiable_ownership_is_preserved() {
    let dir = tempfile::tempdir().unwrap();
    let agents = dir.path().join("AGENTS.md");
    let drifted = "before\n<!-- BEGIN PROVENANCE SKILLS -->\n<!-- Installed by provenance 0.1.0; content hash fnv1a64:0000000000000000 -->\nchanged\n<!-- END PROVENANCE SKILLS -->\nafter\n";
    std::fs::write(&agents, drifted).unwrap();

    install_local(dir.path()).success();

    assert_eq!(std::fs::read_to_string(agents).unwrap(), drifted);
}

#[test]
fn authentic_legacy_agents_block_is_removed() {
    let dir = tempfile::tempdir().unwrap();
    let agents = dir.path().join("AGENTS.md");
    let sources = [
        "---\nname: fork-tournament\ndescription: Run a fork tournament when a shaping session hits a genuine design fork — mutually exclusive directions, expensive to reverse, and the human's preference unknowable without concrete artifacts to react to. Implements the `prototype` resolution method from docs/shaping.md - spawn stance-based agents producing competing artifacts as proposals (phase 1, end session), then present them for human disposal and land the decision as a Resolution (phase 2).\n---\n\n# Fork tournament\n",
        "---\nname: swarm-backtrace\ndescription: Reverse-engineer candidate requirements from an existing codebase with a multi-agent swarm. Use when the user wants to extract, mine, backtrace, or reverse-engineer requirements or rules from existing code, bootstrap a Provenance graph from a legacy system, or asks \"what must be true for this code to be correct\". Lands everything as proposals (promotion_state=proposed) against a commit-pinned source — never as active requirements.\n---\n\n# Swarm backtrace\n",
    ];
    let block = render_legacy_agents_block(&sources);
    std::fs::write(&agents, format!("before\n{block}after\n")).unwrap();

    install_local(dir.path()).success();

    assert_eq!(std::fs::read_to_string(agents).unwrap(), "before\nafter\n");
}

#[test]
fn authentic_four_skill_legacy_agents_block_is_removed() {
    let dir = tempfile::tempdir().unwrap();
    let agents = dir.path().join("AGENTS.md");
    let descriptions = [
        ("shaping", "Guide turn-based requirement shaping in Provenance. Use when a user brings a loose idea, asks to refine requirements, work through open shaping questions, graduate fog, or run the Chart/Work loop against an anchor requirement. Land every resolved decision immediately into the graph."),
        ("fork-tournament", "Run a fork tournament when a shaping session hits a genuine design fork — mutually exclusive directions, expensive to reverse, and the human's preference unknowable without concrete artifacts to react to. Implements the `prototype` resolution method from docs/shaping.md - spawn stance-based agents producing competing artifacts as proposals (phase 1, end session), then present them for human disposal and land the decision as a Resolution (phase 2)."),
        ("swarm-backtrace", "Reverse-engineer candidate requirements from an existing codebase with a multi-agent swarm. Use when the user wants to extract, mine, backtrace, or reverse-engineer requirements or rules from existing code, bootstrap a Provenance graph from a legacy system, or asks \"what must be true for this code to be correct\". Lands everything as proposals (promotion_state=proposed) against a commit-pinned source — never as active requirements."),
        ("grounded-writing", GROUNDED_DESCRIPTION),
    ];
    let sources = descriptions.map(|(name, description)| {
        format!("---\nname: {name}\ndescription: {description}\n---\n\n# {name}\n")
    });
    let source_refs = sources.iter().map(String::as_str).collect::<Vec<_>>();
    let block = render_legacy_agents_block(&source_refs);
    std::fs::write(&agents, format!("before\n{block}after\n")).unwrap();

    install_local(dir.path()).success();

    assert_eq!(std::fs::read_to_string(agents).unwrap(), "before\nafter\n");
}

#[test]
fn authentic_block_preserves_non_utf8_surrounding_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let agents = dir.path().join("AGENTS.md");
    let sources = [format!(
        "---\nname: grounded-writing\ndescription: {GROUNDED_DESCRIPTION}\n---\n\n# Grounded writing\n"
    )];
    let block = render_legacy_agents_block(&[sources[0].as_str()]);
    let mut contents = b"prefix\xff\n".to_vec();
    contents.extend_from_slice(block.as_bytes());
    contents.extend_from_slice(b"suffix\xfe\n");
    std::fs::write(&agents, contents).unwrap();

    install_local(dir.path()).success();

    assert_eq!(std::fs::read(agents).unwrap(), b"prefix\xff\nsuffix\xfe\n");
}

#[test]
fn drifted_authentic_legacy_agents_block_is_preserved() {
    let dir = tempfile::tempdir().unwrap();
    let agents = dir.path().join("AGENTS.md");
    let sources = [
        "---\nname: fork-tournament\ndescription: Run a fork tournament when a shaping session hits a genuine design fork — mutually exclusive directions, expensive to reverse, and the human's preference unknowable without concrete artifacts to react to. Implements the `prototype` resolution method from docs/shaping.md - spawn stance-based agents producing competing artifacts as proposals (phase 1, end session), then present them for human disposal and land the decision as a Resolution (phase 2).\n---\n\n# Fork tournament\n",
        "---\nname: swarm-backtrace\ndescription: Reverse-engineer candidate requirements from an existing codebase with a multi-agent swarm. Use when the user wants to extract, mine, backtrace, or reverse-engineer requirements or rules from existing code, bootstrap a Provenance graph from a legacy system, or asks \"what must be true for this code to be correct\". Lands everything as proposals (promotion_state=proposed) against a commit-pinned source — never as active requirements.\n---\n\n# Swarm backtrace\n",
    ];
    let block = render_legacy_agents_block(&sources).replace("# Swarm", "user edit\n# Swarm");
    let contents = format!("before\n{block}after\n");
    std::fs::write(&agents, &contents).unwrap();

    install_local(dir.path()).success();

    assert_eq!(std::fs::read_to_string(agents).unwrap(), contents);
}

fn render_legacy_agents_block(sources: &[&str]) -> String {
    let source = sources.concat();
    let mut block = format!(
        "<!-- BEGIN PROVENANCE SKILLS -->\n<!-- Installed by provenance 0.1.0; content hash fnv1a64:{} -->\n# Provenance Skills\n\nThese skills are distributed with the provenance CLI and should match the installed binary.\n\nBefore shaping or backtrace work, run `provenance skills install --target agents-md` if skills are absent.\n",
        fnv1a64(&source)
    );
    for source in sources {
        let name = source
            .lines()
            .find_map(|line| line.strip_prefix("name: "))
            .unwrap();
        let body = source.split_once("\n---\n").unwrap().1.trim_start();
        write!(block, "\n## Skill: {name}\n\n{body}").unwrap();
        if !block.ends_with('\n') {
            block.push('\n');
        }
    }
    block.push_str("<!-- END PROVENANCE SKILLS -->\n");
    block
}

fn install_local(path: &Path) -> assert_cmd::assert::Assert {
    Command::cargo_bin("provenance")
        .unwrap()
        .current_dir(path)
        .args(["skills", "install", "--copy"])
        .assert()
}

fn legacy_dir(base: &Path, root: &str, name: &str) -> PathBuf {
    base.join(root).join(name)
}

fn write_managed_skill(dir: &Path, frontmatter: &str, payload: &str) {
    std::fs::create_dir_all(dir).unwrap();
    std::fs::write(
        dir.join("SKILL.md"),
        format!(
            "{frontmatter}<!-- Installed by provenance 0.1.0; content hash fnv1a64:{} -->\n{payload}",
            fnv1a64(&format!("{frontmatter}{payload}"))
        ),
    )
    .unwrap();
}

fn fnv1a64(content: &str) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in content.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}
