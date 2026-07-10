mod core;
mod ideation;
mod operations;
mod shaping;

pub use core::{
    BoundariesCommand, DomainsCommand, EdgesCommand, FogCommand, RequirementsCommand,
    ResolutionsCommand, RulesCommand, ServiceBindingsCommand, ServiceCreateArgs, ServicesCommand,
    SourceRefCommand, SourcesCommand,
};
pub use ideation::{
    ContributionsCommand, IdeationArtifactKind, PromotionDecisionsCommand, ProposalsCommand,
    SchemaCommand, SwarmBacktraceCommand, SynthesisPacketsCommand,
};
pub use operations::{CoverageCommand, DocsCommand, SkillsCommand, WikiCommand};
pub use shaping::{QuestionsCommand, ThreadCommand, TopicsCommand};

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
        command: DocsCommand,
    },
    Wiki {
        #[command(subcommand)]
        command: WikiCommand,
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
    Edges {
        #[command(subcommand)]
        command: EdgesCommand,
    },
    Domains {
        #[command(subcommand)]
        command: DomainsCommand,
    },
    Boundaries {
        #[command(subcommand)]
        command: BoundariesCommand,
    },
    Topics {
        #[command(subcommand)]
        command: TopicsCommand,
    },
    Questions {
        #[command(subcommand)]
        command: QuestionsCommand,
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
    Services {
        #[command(subcommand)]
        command: ServicesCommand,
    },
    ServiceBindings {
        #[command(subcommand)]
        command: ServiceBindingsCommand,
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
    SwarmBacktrace {
        #[command(subcommand)]
        command: SwarmBacktraceCommand,
    },
    Skills {
        #[command(subcommand)]
        command: SkillsCommand,
    },
    Schema {
        #[command(subcommand)]
        command: SchemaCommand,
    },
    Validate {
        artifact: IdeationArtifactKind,
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
