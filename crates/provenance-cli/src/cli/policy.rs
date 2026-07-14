use crate::output::OutputFormat;
use camino::Utf8PathBuf;
use clap::Subcommand;

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
        #[arg(long = "input-type")]
        input_type: Vec<String>,
        #[arg(long = "input-reference")]
        input_reference: Vec<String>,
        #[arg(long = "input-summary")]
        input_summary: Vec<String>,
        #[arg(long)]
        made_by: Option<String>,
        #[arg(long)]
        approved_by: Option<String>,
        #[arg(long)]
        approved_at: Option<i64>,
        #[arg(long)]
        superseded_by: Option<String>,
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
