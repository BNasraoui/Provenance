use anyhow::Context;
use serde::Serialize;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

const BEGIN_MARKER: &str = "<!-- BEGIN PROVENANCE SKILLS -->";
const END_MARKER: &str = "<!-- END PROVENANCE SKILLS -->";
pub const INSTALL_COMMAND: &str = "provenance skills install --target agents-md";

struct EmbeddedSkill {
    directory: &'static str,
    content: &'static str,
}

const EMBEDDED_SKILLS: &[EmbeddedSkill] = &[
    EmbeddedSkill {
        directory: "fork-tournament",
        content: include_str!("../../../skills/fork-tournament/SKILL.md"),
    },
    EmbeddedSkill {
        directory: "swarm-backtrace",
        content: include_str!("../../../skills/swarm-backtrace/SKILL.md"),
    },
];

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum SkillInstallTarget {
    Claude,
    Opencode,
    AgentsMd,
}

impl SkillInstallTarget {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Opencode => "opencode",
            Self::AgentsMd => "agents-md",
        }
    }
}

#[derive(Serialize)]
pub struct SkillSummary {
    name: String,
    description: String,
}

#[derive(Serialize)]
pub struct InstallReport {
    target: &'static str,
    global: bool,
    status: &'static str,
    files: Vec<FileInstallReport>,
}

#[derive(Serialize)]
pub struct SkillInstallStatus {
    pub installed: bool,
    pub install_command: &'static str,
    pub missing_skills: Vec<String>,
}

#[derive(Serialize)]
struct FileInstallReport {
    path: String,
    status: &'static str,
}

pub fn list() -> anyhow::Result<Vec<SkillSummary>> {
    let mut summaries = EMBEDDED_SKILLS
        .iter()
        .map(|skill| {
            Ok(SkillSummary {
                name: skill_name(skill)?.to_string(),
                description: frontmatter_field(skill.content, "description")
                    .context("embedded skill is missing description")?
                    .to_string(),
            })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    summaries.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(summaries)
}

pub fn show(name: &str) -> anyhow::Result<&'static str> {
    for skill in EMBEDDED_SKILLS {
        if skill_name(skill)? == name {
            return Ok(skill.content);
        }
    }
    anyhow::bail!("unknown skill: {name}")
}

pub fn install(
    target: SkillInstallTarget,
    global: bool,
    force: bool,
) -> anyhow::Result<InstallReport> {
    let base = if global {
        home_dir()?
    } else {
        std::env::current_dir()?
    };
    install_at(target, &base, global, force)
}

pub fn install_agents_md_at(base: &Path, force: bool) -> anyhow::Result<()> {
    write_agents_md_section(&base.join("AGENTS.md"), force)?;
    Ok(())
}

pub fn install_status(repo: &Path) -> anyhow::Result<SkillInstallStatus> {
    let agents_md_installed = agents_md_has_marker(&repo.join("AGENTS.md"));
    let mut missing_skills = Vec::new();
    if !agents_md_installed {
        for skill in EMBEDDED_SKILLS {
            if !skill_file_installed(repo, skill) {
                missing_skills.push(skill_name(skill)?.to_string());
            }
        }
    }

    Ok(SkillInstallStatus {
        installed: missing_skills.is_empty(),
        install_command: INSTALL_COMMAND,
        missing_skills,
    })
}

pub fn render_status_markdown(status: &SkillInstallStatus) -> String {
    let installed = if status.installed { "yes" } else { "no" };
    format!(
        "\n## Skills\n- Installed: {installed}\n- Install command: `{}` from the repo root\n",
        status.install_command
    )
}

fn install_at(
    target: SkillInstallTarget,
    base: &Path,
    global: bool,
    force: bool,
) -> anyhow::Result<InstallReport> {
    let files = match target {
        SkillInstallTarget::Claude => install_skill_files(&base.join(".claude/skills"), force)?,
        SkillInstallTarget::Opencode => install_skill_files(&base.join(".agents/skills"), force)?,
        SkillInstallTarget::AgentsMd => {
            let path = if global {
                base.join(".agents/AGENTS.md")
            } else {
                base.join("AGENTS.md")
            };
            vec![write_agents_md_section(&path, force)?]
        }
    };

    Ok(InstallReport {
        target: target.as_str(),
        global,
        status: combined_status(&files),
        files,
    })
}

fn install_skill_files(skills_dir: &Path, force: bool) -> anyhow::Result<Vec<FileInstallReport>> {
    EMBEDDED_SKILLS
        .iter()
        .map(|skill| {
            let path = skills_dir.join(skill.directory).join("SKILL.md");
            write_managed_file(&path, &render_skill_file(skill), force)
        })
        .collect()
}

fn write_agents_md_section(path: &Path, force: bool) -> anyhow::Result<FileInstallReport> {
    let section = render_agents_md_section()?;
    if !path.exists() {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, section)?;
        return Ok(file_report(path, "installed"));
    }

    let current = std::fs::read_to_string(path)?;
    if let Some((start, end)) = managed_section_range(&current)? {
        let existing_section = &current[start..end];
        if existing_section == section {
            return Ok(file_report(path, "unchanged"));
        }
        anyhow::ensure!(
            force,
            "{} exists and differs; rerun with --force to overwrite",
            path.display()
        );
        let updated = format!("{}{}{}", &current[..start], section, &current[end..]);
        std::fs::write(path, updated)?;
        return Ok(file_report(path, "updated"));
    }

    let mut updated = current;
    if !updated.is_empty() && !updated.ends_with('\n') {
        updated.push('\n');
    }
    if !updated.is_empty() {
        updated.push('\n');
    }
    updated.push_str(&section);
    std::fs::write(path, updated)?;
    Ok(file_report(path, "installed"))
}

fn write_managed_file(
    path: &Path,
    contents: &str,
    force: bool,
) -> anyhow::Result<FileInstallReport> {
    if path.exists() {
        let current = std::fs::read_to_string(path)?;
        if current == contents {
            return Ok(file_report(path, "unchanged"));
        }
        anyhow::ensure!(
            force,
            "{} exists and differs; rerun with --force to overwrite",
            path.display()
        );
        std::fs::write(path, contents)?;
        return Ok(file_report(path, "updated"));
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, contents)?;
    Ok(file_report(path, "installed"))
}

fn render_skill_file(skill: &EmbeddedSkill) -> String {
    const FRONTMATTER_START: &str = "---\n";
    const FRONTMATTER_END: &str = "\n---\n";

    let header = provenance_header(skill.content);
    if let Some(rest) = skill.content.strip_prefix(FRONTMATTER_START) {
        if let Some(end) = rest.find(FRONTMATTER_END) {
            let insertion = FRONTMATTER_START.len() + end + FRONTMATTER_END.len();
            return format!(
                "{}{}\n{}",
                &skill.content[..insertion],
                header,
                &skill.content[insertion..]
            );
        }
    }
    format!("{header}\n{}", skill.content)
}

fn render_agents_md_section() -> anyhow::Result<String> {
    let mut source = String::new();
    for skill in EMBEDDED_SKILLS {
        source.push_str(skill.content);
    }

    let mut section = String::new();
    writeln!(section, "{BEGIN_MARKER}")?;
    writeln!(section, "{}", provenance_header(&source))?;
    writeln!(section, "# Provenance Skills")?;
    writeln!(section)?;
    writeln!(
        section,
        "These skills are distributed with the provenance CLI and should match the installed binary."
    )?;
    writeln!(section)?;
    writeln!(
        section,
        "Before shaping or backtrace work, run `{INSTALL_COMMAND}` if skills are absent."
    )?;
    for skill in EMBEDDED_SKILLS {
        writeln!(section)?;
        writeln!(section, "## Skill: {}", skill_name(skill)?)?;
        writeln!(section)?;
        section.push_str(strip_frontmatter(skill.content).trim_start());
        if !section.ends_with('\n') {
            section.push('\n');
        }
    }
    writeln!(section, "{END_MARKER}")?;
    Ok(section)
}

fn provenance_header(content: &str) -> String {
    format!(
        "<!-- Installed by provenance {}; content hash fnv1a64:{} -->",
        env!("CARGO_PKG_VERSION"),
        fnv1a64(content)
    )
}

fn managed_section_range(contents: &str) -> anyhow::Result<Option<(usize, usize)>> {
    let Some(start) = contents.find(BEGIN_MARKER) else {
        return Ok(None);
    };
    let end_marker_start = contents[start..]
        .find(END_MARKER)
        .map(|end| start + end)
        .ok_or_else(|| {
            anyhow::anyhow!("AGENTS.md provenance skills section is missing end marker")
        })?;
    let mut end = end_marker_start + END_MARKER.len();
    if contents[end..].starts_with('\n') {
        end += 1;
    }
    Ok(Some((start, end)))
}

fn combined_status(files: &[FileInstallReport]) -> &'static str {
    if files.iter().any(|file| file.status == "updated") {
        "updated"
    } else if files.iter().any(|file| file.status == "installed") {
        "installed"
    } else {
        "unchanged"
    }
}

fn file_report(path: &Path, status: &'static str) -> FileInstallReport {
    FileInstallReport {
        path: path.display().to_string(),
        status,
    }
}

fn agents_md_has_marker(path: &Path) -> bool {
    std::fs::read_to_string(path).is_ok_and(|contents| contents.contains(BEGIN_MARKER))
}

fn skill_file_installed(repo: &Path, skill: &EmbeddedSkill) -> bool {
    repo.join(".claude/skills")
        .join(skill.directory)
        .join("SKILL.md")
        .exists()
        || repo
            .join(".agents/skills")
            .join(skill.directory)
            .join("SKILL.md")
            .exists()
}

fn skill_name(skill: &EmbeddedSkill) -> anyhow::Result<&'static str> {
    let name =
        frontmatter_field(skill.content, "name").context("embedded skill is missing name")?;
    anyhow::ensure!(
        name == skill.directory,
        "embedded skill name {name} does not match directory {}",
        skill.directory
    );
    Ok(name)
}

fn frontmatter_field<'a>(content: &'a str, field: &str) -> Option<&'a str> {
    let mut lines = content.lines();
    if lines.next()? != "---" {
        return None;
    }
    let prefix = format!("{field}:");
    for line in lines {
        if line == "---" {
            return None;
        }
        if let Some(value) = line.strip_prefix(&prefix) {
            return Some(value.trim());
        }
    }
    None
}

fn strip_frontmatter(content: &str) -> &str {
    const FRONTMATTER_START: &str = "---\n";
    const FRONTMATTER_END: &str = "\n---\n";

    if let Some(rest) = content.strip_prefix(FRONTMATTER_START) {
        if let Some(end) = rest.find(FRONTMATTER_END) {
            return &rest[end + FRONTMATTER_END.len()..];
        }
    }
    content
}

fn fnv1a64(content: &str) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in content.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}

fn home_dir() -> anyhow::Result<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .context("HOME is not set")
}
