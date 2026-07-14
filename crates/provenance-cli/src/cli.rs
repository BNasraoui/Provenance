pub mod graph;
pub mod ideation;
pub mod knowledge;
pub mod policy;
pub mod services;
pub mod shaping;
pub mod workspace;

pub use ideation::{IdeationArtifactKind, SchemaCommand};

use crate::output::OutputFormat;
use camino::Utf8PathBuf;
use clap::{Parser, Subcommand};
use serde::Serialize;

#[derive(Parser)]
pub struct Cli {
    #[arg(long, global = true)]
    pub quiet: bool,
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
    Docs {
        #[command(subcommand)]
        command: workspace::DocsCommand,
    },
    Wiki {
        #[command(subcommand)]
        command: workspace::WikiCommand,
    },
    Materialize {
        #[arg(long, default_value = ".")]
        repo: Utf8PathBuf,
        #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
        format: OutputFormat,
    },
    Sources {
        #[command(subcommand)]
        command: knowledge::SourcesCommand,
    },
    Requirements {
        #[command(subcommand)]
        command: knowledge::RequirementsCommand,
    },
    Edges {
        #[command(subcommand)]
        command: graph::EdgesCommand,
    },
    Domains {
        #[command(subcommand)]
        command: knowledge::DomainsCommand,
    },
    Boundaries {
        #[command(subcommand)]
        command: knowledge::BoundariesCommand,
    },
    Topics {
        #[command(subcommand)]
        command: shaping::TopicsCommand,
    },
    Questions {
        #[command(subcommand)]
        command: shaping::QuestionsCommand,
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
        command: policy::ResolutionsCommand,
    },
    Rules {
        #[command(subcommand)]
        command: policy::RulesCommand,
    },
    Services {
        #[command(subcommand)]
        command: services::ServicesCommand,
    },
    ServiceBindings {
        #[command(subcommand)]
        command: services::ServiceBindingsCommand,
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
        command: shaping::ThreadCommand,
    },
    Contributions {
        #[command(subcommand)]
        command: ideation::ContributionsCommand,
    },
    SynthesisPackets {
        #[command(subcommand)]
        command: ideation::SynthesisPacketsCommand,
    },
    Proposals {
        #[command(subcommand)]
        command: ideation::ProposalsCommand,
    },
    PromotionDecisions {
        #[command(subcommand)]
        command: ideation::PromotionDecisionsCommand,
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
        command: workspace::CoverageCommand,
    },
    SwarmBacktrace {
        #[command(subcommand)]
        command: ideation::SwarmBacktraceCommand,
    },
    Skills {
        #[command(subcommand)]
        command: workspace::SkillsCommand,
    },
    Schema {
        #[command(subcommand)]
        command: ideation::SchemaCommand,
    },
    Validate {
        artifact: ideation::IdeationArtifactKind,
        #[arg(long)]
        input: Utf8PathBuf,
        #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
        format: OutputFormat,
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

#[derive(Serialize)]
pub struct Status<'a> {
    pub status: &'a str,
}
