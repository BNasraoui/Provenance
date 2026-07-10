use anyhow::Context;
use std::path::Path;

const LEGACY_SKILL_DIRECTORIES: &[&str] = &[
    "shaping",
    "fork-tournament",
    "swarm-backtrace",
    "grounded-writing",
];
const BEGIN_MARKER: &str = "<!-- BEGIN PROVENANCE SKILLS -->";
const END_MARKER: &str = "<!-- END PROVENANCE SKILLS -->";
const HEADER_PREFIX: &str = "<!-- Installed by provenance ";
const HASH_PREFIX: &str = "; content hash fnv1a64:";
const AGENTS_PREAMBLE: &str = "# Provenance Skills\n\nThese skills are distributed with the provenance CLI and should match the installed binary.\n\nBefore shaping or backtrace work, run `provenance skills install --target agents-md` if skills are absent.\n";

pub fn cleanup(base: &Path, global: bool) -> anyhow::Result<()> {
    for root in [base.join(".claude/skills"), base.join(".agents/skills")] {
        for directory in LEGACY_SKILL_DIRECTORIES {
            cleanup_skill_dir(&root.join(directory))?;
        }
    }

    let agents = if global {
        base.join(".agents/AGENTS.md")
    } else {
        base.join("AGENTS.md")
    };
    cleanup_agents(&agents)
}

fn cleanup_skill_dir(path: &Path) -> anyhow::Result<()> {
    let Ok(metadata) = std::fs::symlink_metadata(path) else {
        return Ok(());
    };
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Ok(());
    }
    let skill_file = path.join("SKILL.md");
    let Ok(contents) = std::fs::read_to_string(&skill_file) else {
        return Ok(());
    };
    if !valid_managed_skill(&contents) {
        return Ok(());
    }

    std::fs::remove_file(&skill_file).with_context(|| {
        format!(
            "failed to remove legacy skill file {}",
            skill_file.display()
        )
    })?;
    if std::fs::read_dir(path)?.next().is_none() {
        std::fs::remove_dir(path).with_context(|| {
            format!(
                "failed to remove empty legacy skill directory {}",
                path.display()
            )
        })?;
    }
    Ok(())
}

fn valid_managed_skill(contents: &str) -> bool {
    let Some(frontmatter_end) = contents
        .strip_prefix("---\n")
        .and_then(|rest| rest.find("\n---\n"))
        .map(|end| end + "---\n".len() + "\n---\n".len())
    else {
        return false;
    };
    let Some(after_header) = contents[frontmatter_end..].find('\n') else {
        return false;
    };
    let header_end = frontmatter_end + after_header;
    let header = &contents[frontmatter_end..header_end];
    let Some(expected_hash) = header_hash(header) else {
        return false;
    };
    let payload = &contents[header_end + 1..];
    let original = format!("{}{payload}", &contents[..frontmatter_end]);
    fnv1a64(&original) == expected_hash
}

fn cleanup_agents(path: &Path) -> anyhow::Result<()> {
    let Ok(contents) = std::fs::read_to_string(path) else {
        return Ok(());
    };
    let Some(start) = contents.find(BEGIN_MARKER) else {
        return Ok(());
    };
    let block = &contents[start..];
    let Some(end_offset) = block.find(END_MARKER) else {
        return Ok(());
    };
    let marker_line_end = BEGIN_MARKER.len() + 1;
    if !block.starts_with(&format!("{BEGIN_MARKER}\n")) {
        return Ok(());
    }
    let Some(header_line_end) = block[marker_line_end..].find('\n') else {
        return Ok(());
    };
    let header_end = marker_line_end + header_line_end;
    let Some(expected_hash) = header_hash(&block[marker_line_end..header_end]) else {
        return Ok(());
    };
    let payload = &block[header_end + 1..end_offset];
    if legacy_agents_source_hash(payload).as_deref() != Some(expected_hash) {
        return Ok(());
    }
    let mut end = start + end_offset + END_MARKER.len();
    if contents[end..].starts_with('\n') {
        end += 1;
    }
    let updated = format!("{}{}", &contents[..start], &contents[end..]);
    std::fs::write(path, updated).with_context(|| {
        format!(
            "failed to remove legacy skill section from {}",
            path.display()
        )
    })
}

fn legacy_agents_source_hash(payload: &str) -> Option<String> {
    let sections = payload.strip_prefix(AGENTS_PREAMBLE)?;
    let sections = sections.strip_prefix('\n')?;
    let mut source = String::new();
    for section in sections.split("\n## Skill: ") {
        let section = section.strip_prefix("## Skill: ").unwrap_or(section);
        let (name, body) = section.split_once("\n\n")?;
        let description = legacy_description(name)?;
        source.push_str("---\nname: ");
        source.push_str(name);
        source.push_str("\ndescription: ");
        source.push_str(description);
        source.push_str("\n---\n\n");
        source.push_str(body);
    }
    Some(fnv1a64(&source))
}

fn legacy_description(name: &str) -> Option<&'static str> {
    match name {
        "fork-tournament" => Some("Run a fork tournament when a shaping session hits a genuine design fork — mutually exclusive directions, expensive to reverse, and the human's preference unknowable without concrete artifacts to react to. Implements the `prototype` resolution method from docs/shaping.md - spawn stance-based agents producing competing artifacts as proposals (phase 1, end session), then present them for human disposal and land the decision as a Resolution (phase 2)."),
        "shaping" => Some("Guide turn-based requirement shaping in Provenance. Use when a user brings a loose idea, asks to refine requirements, work through open shaping questions, graduate fog, or run the Chart/Work loop against an anchor requirement. Land every resolved decision immediately into the graph."),
        "swarm-backtrace" => Some("Reverse-engineer candidate requirements from an existing codebase with a multi-agent swarm. Use when the user wants to extract, mine, backtrace, or reverse-engineer requirements or rules from existing code, bootstrap a Provenance graph from a legacy system, or asks \"what must be true for this code to be correct\". Lands everything as proposals (promotion_state=proposed) against a commit-pinned source — never as active requirements."),
        _ => None,
    }
}

fn header_hash(header: &str) -> Option<&str> {
    let header = header.strip_prefix(HEADER_PREFIX)?;
    let (version, hash) = header.split_once(HASH_PREFIX)?;
    let hash = hash.strip_suffix(" -->")?;
    (!version.is_empty() && hash.len() == 16 && hash.bytes().all(|b| b.is_ascii_hexdigit()))
        .then_some(hash)
}

fn fnv1a64(content: &str) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in content.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}
