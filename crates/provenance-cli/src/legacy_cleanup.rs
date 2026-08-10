use crate::skills::stamp::{fnv1a64, header_hash};
use crate::skills::FileStatus;
use anyhow::Context;
use provenance_macros::rule;
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
const AGENTS_PREAMBLE: &str = "# Provenance Skills\n\nThese skills are distributed with the provenance CLI and should match the installed binary.\n\nBefore shaping or backtrace work, run `provenance skills install --target agents-md` if skills are absent.\n";

pub struct CleanupChange {
    pub path: PathBuf,
    pub status: FileStatus,
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
            status: FileStatus::Updated,
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
        status: FileStatus::Removed,
    }))
}

/// A legacy skill file may be deleted only when its own stamp proves it is
/// byte-for-byte what provenance installed. The stamp is one header line
/// sitting immediately after the frontmatter, carrying the hash of the file
/// as installed (frontmatter and payload, header line removed). Anything
/// else - a header elsewhere in the file, a malformed header, an edited
/// payload, a missing frontmatter - belongs to the user and is left alone.
#[rule("rule_legacy_cleanup_ownership")]
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
    let payload = &contents[header_end + 1..];
    let installed = format!("{}{payload}", &contents[..frontmatter_end]);
    hash_proves_ownership(header, &installed)
}

/// The proof shared by both cleanups: the hash a header claims must equal
/// the hash of `installed`, the exact text provenance would have written.
/// Any user edit to that text breaks the match, so nothing gets destroyed.
fn hash_proves_ownership(header: &str, installed: &str) -> bool {
    header_hash(header) == Some(fnv1a64(installed).as_str())
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
    let Ok(payload) = std::str::from_utf8(&block[header_end + 1..end_offset]) else {
        return Ok(false);
    };
    let Some(installed) = legacy_agents_source(payload) else {
        return Ok(false);
    };
    if !hash_proves_ownership(header, &installed) {
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

/// Rebuilds the skill sources a legacy AGENTS.md block was rendered from,
/// so the block can be hashed against the header the installer stamped.
fn legacy_agents_source(payload: &str) -> Option<String> {
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
    Some(source)
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

#[cfg(test)]
mod tests {
    use std::fmt::Write as _;

    use provenance_macros::verifies;

    use super::*;

    /// `SplitMix64`. The seeds below are fixed, so every run walks the same
    /// generated files and a failure is reproducible from the test name.
    struct Rng(u64);

    impl Rng {
        fn next_u64(&mut self) -> u64 {
            self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
            let mut word = self.0;
            word = (word ^ (word >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            word = (word ^ (word >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
            word ^ (word >> 31)
        }

        fn below(&mut self, bound: usize) -> usize {
            usize::try_from(self.next_u64() % bound as u64).unwrap()
        }

        fn pick(&mut self, choices: &[&'static str]) -> &'static str {
            choices[self.below(choices.len())]
        }
    }

    const FRONTMATTER_VALUES: &[&str] = &[
        "old",
        "a name",
        "0.1.0",
        "with - dash",
        "unicode é",
        "日本語",
        "<!-- not a header -->",
        "trailing space ",
        "",
    ];

    const ASCII_FRONTMATTER_VALUES: &[&str] = &[
        "old",
        "a name",
        "0.1.0",
        "with - dash",
        "<!-- not a header -->",
        "trailing space ",
        "",
    ];

    // Payload chunks worth stressing: blank lines, frontmatter delimiters
    // loose in the body, multibyte characters that move every byte offset
    // after them, and a decoy header line the parser must ignore because it
    // is not the one sitting right after the frontmatter.
    const PAYLOAD_CHUNKS: &[&str] = &[
        "a",
        " ",
        "\n",
        "---\n",
        "\n---\n",
        "# Heading\n",
        "body text ",
        "é",
        "日本語\n",
        "<!--",
        "-->",
        "\r\n",
        "<!-- Installed by provenance 9.9.9; content hash fnv1a64:0000000000000000 -->\n",
    ];

    const ASCII_CHUNKS: &[&str] = &[
        "a",
        " ",
        "\n",
        "---\n",
        "# Heading\n",
        "body text ",
        "<!--",
        "-->",
    ];

    fn gen_frontmatter(rng: &mut Rng, values: &[&'static str]) -> String {
        let mut text = String::from("---\n");
        for index in 0..=rng.below(3) {
            let value = rng.pick(values);
            writeln!(text, "key{index}: {value}").unwrap();
        }
        text.push_str("---\n");
        text
    }

    fn gen_payload(rng: &mut Rng, chunks: &[&'static str], min_bytes: usize) -> String {
        let mut text = String::new();
        while text.len() < min_bytes {
            text.push_str(rng.pick(chunks));
        }
        text
    }

    fn gen_version(rng: &mut Rng) -> String {
        format!("{}.{}.{}", rng.below(10), rng.below(30), rng.below(100))
    }

    /// Independent restatement of the stamp the installer writes, kept apart
    /// from the parser's own constants: one header line carrying the hash of
    /// the file as installed, placed straight after the frontmatter.
    fn header_line(version: &str, installed: &str) -> String {
        format!(
            "<!-- Installed by provenance {version}; content hash fnv1a64:{} -->",
            fnv1a64(installed)
        )
    }

    fn stamp(frontmatter: &str, version: &str, payload: &str) -> String {
        let header = header_line(version, &format!("{frontmatter}{payload}"));
        format!("{frontmatter}{header}\n{payload}")
    }

    #[test]
    #[verifies("rule_legacy_cleanup_ownership", property)]
    fn accepts_every_correctly_stamped_file() {
        let mut rng = Rng(0x5eed_0001);
        for _ in 0..2048 {
            let frontmatter = gen_frontmatter(&mut rng, FRONTMATTER_VALUES);
            let payload = gen_payload(&mut rng, PAYLOAD_CHUNKS, 64);
            let contents = stamp(&frontmatter, &gen_version(&mut rng), &payload);

            assert!(
                valid_managed_skill(&contents),
                "refused to own a file stamped exactly as installed: {contents:?}"
            );
        }
    }

    #[test]
    #[verifies("rule_legacy_cleanup_ownership", property)]
    fn rejects_every_single_byte_edit_to_the_installed_bytes() {
        let mut rng = Rng(0x5eed_0002);
        for _ in 0..24 {
            let frontmatter = gen_frontmatter(&mut rng, ASCII_FRONTMATTER_VALUES);
            let payload = gen_payload(&mut rng, ASCII_CHUNKS, 40);
            let contents = stamp(&frontmatter, &gen_version(&mut rng), &payload);
            // Everything the hash covers: the frontmatter and the payload,
            // which is the whole file bar the header line the installer added.
            let payload_start = contents.len() - payload.len();
            let installed = (0..frontmatter.len()).chain(payload_start..contents.len());

            for offset in installed {
                let original = contents.as_bytes()[offset];
                for replacement in 0..=0x7f_u8 {
                    if replacement == original {
                        continue;
                    }
                    let mut bytes = contents.as_bytes().to_vec();
                    bytes[offset] = replacement;
                    let edited = String::from_utf8(bytes).unwrap();

                    assert!(
                        !valid_managed_skill(&edited),
                        "claimed a file edited at byte {offset}: {edited:?}"
                    );
                }
            }
        }
    }

    #[test]
    #[verifies("rule_legacy_cleanup_ownership", property)]
    fn rejects_every_header_off_its_required_placement() {
        let mut rng = Rng(0x5eed_0003);
        for _ in 0..1024 {
            let frontmatter = gen_frontmatter(&mut rng, FRONTMATTER_VALUES);
            let payload = gen_payload(&mut rng, PAYLOAD_CHUNKS, 64);
            let header = header_line(&gen_version(&mut rng), &format!("{frontmatter}{payload}"));
            let body = frontmatter.strip_prefix("---\n").unwrap();
            let misplaced = [
                (
                    "above the frontmatter",
                    format!("{header}\n{frontmatter}{payload}"),
                ),
                (
                    "inside the frontmatter",
                    format!("---\n{header}\n{body}{payload}"),
                ),
                (
                    "one line late",
                    format!("{frontmatter}decoy line\n{header}\n{payload}"),
                ),
                (
                    "below the payload",
                    format!("{frontmatter}{payload}{header}\n"),
                ),
                ("absent", format!("{frontmatter}{payload}")),
            ];

            for (placement, contents) in misplaced {
                assert!(
                    !valid_managed_skill(&contents),
                    "claimed a file with its header {placement}: {contents:?}"
                );
            }
        }
    }
}
