use crate::output::{self, OutputFormat};
use crate::skills;
use camino::Utf8PathBuf;
use provenance_core::ScopeId;
use provenance_store::{cache, layout::ProvenanceLayout};

#[derive(serde::Serialize)]
struct PrimeOutput {
    #[serde(flatten)]
    view: cache::PrimeContextView,
    skills: skills::SkillInstallStatus,
}

pub(super) fn handle(
    repo: Utf8PathBuf,
    scope: String,
    format: OutputFormat,
    include_threads: bool,
) -> anyhow::Result<()> {
    let skill_status = skills::install_status(repo.as_std_path())?;
    let view = cache::prime_context(
        &ProvenanceLayout::new(repo),
        &ScopeId::new(scope)?,
        include_threads,
    )?;
    if matches!(format, OutputFormat::Markdown | OutputFormat::Toon) {
        let mut rendered = cache::render_prime_markdown(&view);
        rendered.push_str(&skills::render_status_markdown(&skill_status));
        println!("{rendered}");
    } else {
        output::print(
            format,
            &PrimeOutput {
                view,
                skills: skill_status,
            },
        )?;
    }
    Ok(())
}
