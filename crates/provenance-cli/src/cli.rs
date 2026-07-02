use crate::output::OutputFormat;
use camino::Utf8PathBuf;
use clap::{Parser, Subcommand};
use serde::Serialize;

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Init {
        #[arg(long)]
        path: Utf8PathBuf,
        #[arg(long)]
        scope: String,
        #[arg(long, default_value = ".")]
        path_prefix: Utf8PathBuf,
    },
    Check {
        #[arg(long, default_value = ".")]
        repo: Utf8PathBuf,
        #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
        format: OutputFormat,
    },
    Materialize {
        #[arg(long, default_value = ".")]
        repo: Utf8PathBuf,
        #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
        format: OutputFormat,
    },
    Sources {
        #[command(subcommand)]
        command: SourcesCommand,
    },
    Requirements {
        #[command(subcommand)]
        command: RequirementsCommand,
    },
    Graph {
        requirement_id: String,
        #[arg(long, default_value = ".")]
        repo: Utf8PathBuf,
        #[arg(long, default_value = "default")]
        scope: String,
        #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
        format: OutputFormat,
    },
    Resolutions {
        #[command(subcommand)]
        command: ResolutionsCommand,
    },
    Rules {
        #[command(subcommand)]
        command: RulesCommand,
    },
    Traceability {
        rule_id: String,
        #[arg(long, default_value = ".")]
        repo: Utf8PathBuf,
        #[arg(long, default_value = "default")]
        scope: String,
        #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
        format: OutputFormat,
    },
    Gaps {
        #[arg(long, default_value = ".")]
        repo: Utf8PathBuf,
        #[arg(long, default_value = "default")]
        scope: String,
        #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
        format: OutputFormat,
    },
    Thread {
        #[command(subcommand)]
        command: ThreadCommand,
    },
    Contributions {
        #[command(subcommand)]
        command: ContributionsCommand,
    },
    SynthesisPackets {
        #[command(subcommand)]
        command: SynthesisPacketsCommand,
    },
    Proposals {
        #[command(subcommand)]
        command: ProposalsCommand,
    },
    PromotionDecisions {
        #[command(subcommand)]
        command: PromotionDecisionsCommand,
    },
    Prime {
        #[arg(long, default_value = ".")]
        repo: Utf8PathBuf,
        #[arg(long, default_value = "default")]
        scope: String,
        #[arg(long, value_enum, default_value_t = OutputFormat::Markdown)]
        format: OutputFormat,
        #[arg(long)]
        include_threads: bool,
    },
    Impact {
        id: String,
        #[arg(long, default_value = ".")]
        repo: Utf8PathBuf,
        #[arg(long, default_value = "default")]
        scope: String,
        #[arg(long)]
        node_type: String,
        #[arg(long, default_value_t = 3)]
        max_hops: u32,
        #[arg(long)]
        follow_indirect: bool,
        #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
        format: OutputFormat,
    },
    Stale {
        #[arg(long, default_value = ".")]
        repo: Utf8PathBuf,
        #[arg(long, default_value = "default")]
        scope: String,
        #[arg(long, default_value_t = 0)]
        min_age_days: u32,
        #[arg(long)]
        rule_severities: Option<String>,
        #[arg(long, default_value_t = 0)]
        min_downstream_rules: u32,
        #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
        format: OutputFormat,
    },
    Health {
        #[arg(long, default_value = ".")]
        repo: Utf8PathBuf,
        #[arg(long, default_value = "default")]
        scope: String,
        #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
        format: OutputFormat,
    },
    Orphans {
        #[arg(long, default_value = ".")]
        repo: Utf8PathBuf,
        #[arg(long, default_value = "default")]
        scope: String,
        #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
        format: OutputFormat,
    },
    Coverage {
        #[command(subcommand)]
        command: CoverageCommand,
    },
    Export {
        #[arg(long, default_value = ".")]
        repo: Utf8PathBuf,
        #[arg(long, default_value = "default")]
        scope: String,
        #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
        format: OutputFormat,
        #[arg(long)]
        output: Option<Utf8PathBuf>,
    },
    Import {
        #[arg(long, default_value = ".")]
        repo: Utf8PathBuf,
        #[arg(long, default_value = "default")]
        scope: String,
        #[arg(long)]
        input: Utf8PathBuf,
        #[arg(long)]
        dry_run: bool,
        #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
        format: OutputFormat,
    },
    MergeJsonl {
        base: Utf8PathBuf,
        ours: Utf8PathBuf,
        theirs: Utf8PathBuf,
        #[arg(long)]
        output: Option<Utf8PathBuf>,
        #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
        format: OutputFormat,
    },
}

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum ContributionsCommand {
    Create {
        #[arg(long, default_value = ".")]
        repo: Utf8PathBuf,
        #[arg(long)]
        scope: String,
        #[arg(long)]
        id: String,
        #[arg(long)]
        target_type: String,
        #[arg(long)]
        target_id: String,
        #[arg(long)]
        participant_slot: String,
        #[arg(long)]
        stance: String,
        #[arg(long)]
        strongest_finding: String,
        #[arg(long, default_value = "[]")]
        evidence_json: String,
        #[arg(long, default_value = "[]")]
        claims_json: String,
        #[arg(long, default_value = "[]")]
        risks_json: String,
        #[arg(long, default_value = "[]")]
        objections_json: String,
        #[arg(long, default_value = "[]")]
        challenges_json: String,
        #[arg(long, default_value = "[]")]
        suggested_changes_json: String,
        #[arg(long, default_value = "[]")]
        unsupported_recommendations_json: String,
        #[arg(long, default_value = "medium")]
        uncertainty_level: String,
        #[arg(long)]
        uncertainty_rationale: String,
        #[arg(long, default_value = "[]")]
        open_questions_json: String,
        #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
        format: OutputFormat,
    },
    List {
        #[arg(long, default_value = ".")]
        repo: Utf8PathBuf,
        #[arg(long)]
        scope: String,
        #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
        format: OutputFormat,
    },
}

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum SynthesisPacketsCommand {
    Create {
        #[arg(long, default_value = ".")]
        repo: Utf8PathBuf,
        #[arg(long)]
        scope: String,
        #[arg(long)]
        id: String,
        #[arg(long)]
        target_type: String,
        #[arg(long)]
        target_id: String,
        #[arg(long)]
        summary: String,
        #[arg(long, default_value = "[]")]
        consensus_json: String,
        #[arg(long, default_value = "[]")]
        contested_claims_json: String,
        #[arg(long, default_value = "[]")]
        minority_objections_json: String,
        #[arg(long, default_value = "[]")]
        evidence_gaps_json: String,
        #[arg(long, default_value = "[]")]
        unsupported_speculation_json: String,
        #[arg(long, default_value = "[]")]
        open_questions_json: String,
        #[arg(long, default_value = "[]")]
        suggested_artifacts_json: String,
        #[arg(long, default_value = "[]")]
        required_human_decisions_json: String,
        #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
        format: OutputFormat,
    },
    List {
        #[arg(long, default_value = ".")]
        repo: Utf8PathBuf,
        #[arg(long)]
        scope: String,
        #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
        format: OutputFormat,
    },
}

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum ProposalsCommand {
    Create {
        #[arg(long, default_value = ".")]
        repo: Utf8PathBuf,
        #[arg(long)]
        scope: String,
        #[arg(long)]
        id: String,
        #[arg(long)]
        proposal_key: String,
        #[arg(long)]
        proposal_type: String,
        #[arg(long)]
        title: String,
        #[arg(long)]
        summary: String,
        #[arg(long)]
        target_type: String,
        #[arg(long)]
        target_id: String,
        #[arg(long)]
        source_id: Vec<String>,
        #[arg(long, default_value = "[]")]
        evidence_json: String,
        #[arg(long)]
        supporting_claim_id: Vec<String>,
        #[arg(long, default_value = "proposed")]
        promotion_state: String,
        #[arg(long)]
        duplicate_of: Option<String>,
        #[arg(long)]
        superseded_by: Option<String>,
        #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
        format: OutputFormat,
    },
    List {
        #[arg(long, default_value = ".")]
        repo: Utf8PathBuf,
        #[arg(long)]
        scope: String,
        #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
        format: OutputFormat,
    },
}

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum PromotionDecisionsCommand {
    Create {
        #[arg(long, default_value = ".")]
        repo: Utf8PathBuf,
        #[arg(long)]
        scope: String,
        #[arg(long)]
        id: String,
        #[arg(long)]
        proposal_id: String,
        #[arg(long)]
        decision: String,
        #[arg(long)]
        rationale: String,
        #[arg(long)]
        actor_id: String,
        #[arg(long)]
        actor_type: String,
        #[arg(long)]
        actor_name: Option<String>,
        #[arg(long)]
        canonical_artifact_type: Option<String>,
        #[arg(long)]
        canonical_artifact_id: Option<String>,
        #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
        format: OutputFormat,
    },
    List {
        #[arg(long, default_value = ".")]
        repo: Utf8PathBuf,
        #[arg(long)]
        scope: String,
        #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
        format: OutputFormat,
    },
}

#[derive(Subcommand)]
pub enum CoverageCommand {
    Scan {
        #[arg(long, default_value = ".")]
        repo: Utf8PathBuf,
        #[arg(long)]
        path: Utf8PathBuf,
        #[arg(long, default_value = "default")]
        scope: String,
        #[arg(long)]
        validate_rules: bool,
        #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
        format: OutputFormat,
        #[arg(long)]
        output: Option<Utf8PathBuf>,
    },
}

#[derive(Subcommand)]
pub enum ThreadCommand {
    Post {
        #[arg(long, default_value = ".")]
        repo: Utf8PathBuf,
        #[arg(long)]
        scope: String,
        #[arg(long)]
        parent_type: String,
        #[arg(long)]
        parent_id: String,
        #[arg(long)]
        role: String,
        body: String,
        #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
        format: OutputFormat,
    },
    List {
        #[arg(long, default_value = ".")]
        repo: Utf8PathBuf,
        #[arg(long)]
        scope: String,
        #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
        format: OutputFormat,
    },
}

#[derive(Subcommand)]
pub enum ResolutionsCommand {
    Create {
        #[arg(long, default_value = ".")]
        repo: Utf8PathBuf,
        #[arg(long)]
        scope: String,
        #[arg(long)]
        id: String,
        #[arg(long)]
        title: String,
        #[arg(long)]
        requirement_id: Option<String>,
        #[arg(long)]
        position: String,
        #[arg(long)]
        rationale: String,
        #[arg(long, default_value = "proposed")]
        status: String,
        #[arg(long)]
        context: Option<String>,
        #[arg(long)]
        enforcement: Option<String>,
        #[arg(long)]
        confidence: Option<f64>,
        #[arg(long)]
        origin_thread: Option<String>,
        #[arg(long)]
        origin_message: Option<String>,
        #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
        format: OutputFormat,
    },
}

#[derive(Subcommand)]
pub enum RulesCommand {
    Create {
        #[arg(long, default_value = ".")]
        repo: Utf8PathBuf,
        #[arg(long)]
        scope: String,
        #[arg(long)]
        id: String,
        #[arg(long)]
        rule_code: String,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        requirement_id: Option<String>,
        #[arg(long)]
        resolution_id: Option<String>,
        #[arg(long)]
        statement: String,
        #[arg(long, default_value = "active")]
        status: String,
        #[arg(long, default_value = "medium")]
        severity: String,
        #[arg(long)]
        rule_type: Option<String>,
        #[arg(long)]
        modality: Option<String>,
        #[arg(long)]
        confidence: Option<f64>,
        #[arg(long)]
        extraction_method: Option<String>,
        #[arg(long)]
        source_document: Option<String>,
        #[arg(long)]
        source_section: Option<String>,
        #[arg(long)]
        origin_thread: Option<String>,
        #[arg(long)]
        origin_message: Option<String>,
        #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
        format: OutputFormat,
    },
}

#[derive(Subcommand)]
pub enum SourcesCommand {
    Create {
        #[arg(long, default_value = ".")]
        repo: Utf8PathBuf,
        #[arg(long)]
        scope: String,
        #[arg(long)]
        id: String,
        #[arg(long)]
        name: String,
        #[arg(long, default_value = "policy")]
        source_type: String,
        #[arg(long)]
        url: Option<String>,
        #[arg(long)]
        reference: Option<String>,
        #[arg(long)]
        origin_thread: Option<String>,
        #[arg(long)]
        origin_message: Option<String>,
        #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
        format: OutputFormat,
    },
}

#[derive(Subcommand)]
pub enum RequirementsCommand {
    Create {
        #[arg(long, default_value = ".")]
        repo: Utf8PathBuf,
        #[arg(long)]
        scope: String,
        #[arg(long)]
        id: String,
        #[arg(long)]
        statement: String,
        #[arg(long)]
        description: Option<String>,
        #[arg(long, default_value = "active")]
        status: String,
        #[arg(long)]
        origin_thread: Option<String>,
        #[arg(long)]
        origin_message: Option<String>,
        #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
        format: OutputFormat,
    },
    SourceRef {
        #[command(subcommand)]
        command: SourceRefCommand,
    },
}

#[derive(Subcommand)]
pub enum SourceRefCommand {
    Add {
        #[arg(long, default_value = ".")]
        repo: Utf8PathBuf,
        #[arg(long)]
        scope: String,
        #[arg(long)]
        requirement_id: String,
        #[arg(long)]
        source_id: String,
        #[arg(long)]
        clause: Option<String>,
        #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
        format: OutputFormat,
    },
}

#[derive(Serialize)]
pub struct Status<'a> {
    pub status: &'a str,
}
