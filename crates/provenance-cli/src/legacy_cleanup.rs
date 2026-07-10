use anyhow::Context;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

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

pub struct CleanupChange {
    pub path: PathBuf,
    pub status: &'static str,
}

pub fn cleanup(base: &Path, global: bool) -> anyhow::Result<Vec<CleanupChange>> {
    let mut changes = Vec::new();
    for root in [base.join(".claude/skills"), base.join(".agents/skills")] {
        for directory in LEGACY_SKILL_DIRECTORIES {
            if let Some(change) = cleanup_skill_dir(&root.join(directory))? {
                changes.push(change);
            }
        }
    }

    let agents = if global {
        base.join(".agents/AGENTS.md")
    } else {
        base.join("AGENTS.md")
    };
    if cleanup_agents(&agents)? {
        changes.push(CleanupChange {
            path: agents,
            status: "updated",
        });
    }
    Ok(changes)
}

fn cleanup_skill_dir(path: &Path) -> anyhow::Result<Option<CleanupChange>> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Ok(None);
    }
    let skill_file = path.join("SKILL.md");
    let contents = match std::fs::read_to_string(&skill_file) {
        Ok(contents) => contents,
        Err(error) if matches!(error.kind(), ErrorKind::NotFound | ErrorKind::InvalidData) => {
            return Ok(None);
        }
        Err(error) => return Err(error.into()),
    };
    if !valid_managed_skill(&contents) {
        return Ok(None);
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
    Ok(Some(CleanupChange {
        path: skill_file,
        status: "removed",
    }))
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

fn cleanup_agents(path: &Path) -> anyhow::Result<bool> {
    let contents = match std::fs::read(path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error.into()),
    };
    let begin = BEGIN_MARKER.as_bytes();
    let end_marker = END_MARKER.as_bytes();
    let Some(start) = find_bytes(&contents, begin) else {
        return Ok(false);
    };
    let block = &contents[start..];
    let Some(end_offset) = find_bytes(block, end_marker) else {
        return Ok(false);
    };
    let marker_line_end = BEGIN_MARKER.len() + 1;
    if !block.starts_with(format!("{BEGIN_MARKER}\n").as_bytes()) {
        return Ok(false);
    }
    let Some(header_line_end) = block[marker_line_end..]
        .iter()
        .position(|byte| *byte == b'\n')
    else {
        return Ok(false);
    };
    let header_end = marker_line_end + header_line_end;
    let Ok(header) = std::str::from_utf8(&block[marker_line_end..header_end]) else {
        return Ok(false);
    };
    let Some(expected_hash) = header_hash(header) else {
        return Ok(false);
    };
    let Ok(payload) = std::str::from_utf8(&block[header_end + 1..end_offset]) else {
        return Ok(false);
    };
    if legacy_agents_source_hash(payload).as_deref() != Some(expected_hash) {
        return Ok(false);
    }
    let mut end = start + end_offset + END_MARKER.len();
    if contents.get(end) == Some(&b'\n') {
        end += 1;
    }
    let mut updated = contents[..start].to_vec();
    updated.extend_from_slice(&contents[end..]);
    atomic_replace(path, &updated).with_context(|| {
        format!(
            "failed to remove legacy skill section from {}",
            path.display()
        )
    })?;
    Ok(true)
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn atomic_replace(path: &Path, contents: &[u8]) -> std::io::Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    for attempt in 0..100_u8 {
        let temporary = parent.join(format!(
            ".{name}.provenance-{}-{attempt}.tmp",
            std::process::id()
        ));
        let mut file = match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
        {
            Ok(file) => file,
            Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        };
        let result = (|| {
            file.write_all(contents)?;
            file.sync_all()?;
            std::fs::set_permissions(&temporary, std::fs::metadata(path)?.permissions())?;
            std::fs::rename(&temporary, path)
        })();
        if result.is_err() {
            let _ = std::fs::remove_file(&temporary);
        }
        return result;
    }
    Err(std::io::Error::new(
        ErrorKind::AlreadyExists,
        "could not allocate temporary file",
    ))
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
        "grounded-writing" => Some("Write specific, evidence-grounded statements for requirements, rules, sources, resolutions, and boundaries — not generic capability language. Use before calling `requirements create/update`, `rules create/update`, `sources create/update`, `resolutions create/update`, or `boundaries create`, especially for a root or mid-level requirement, a statement merging several candidates, or a resolution's position and rationale."),
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
