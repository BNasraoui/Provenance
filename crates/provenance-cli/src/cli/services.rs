use crate::output::OutputFormat;
use camino::Utf8PathBuf;
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct ServiceCreateArgs {
    #[arg(long, default_value = ".")]
    pub repo: Utf8PathBuf,
    #[arg(long)]
    pub scope: String,
    #[arg(long)]
    pub id: String,
    #[arg(long)]
    pub name: String,
    #[arg(long)]
    pub description: Option<String>,
    #[arg(long)]
    pub owner: Option<String>,
    #[arg(long)]
    pub repository: Option<String>,
    #[arg(long)]
    pub environment: Option<String>,
    #[arg(long)]
    pub tier: Option<String>,
    #[arg(long)]
    pub external_id: Option<String>,
    #[arg(long, default_value = "active")]
    pub status: String,
    #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
    pub format: OutputFormat,
}

#[derive(Subcommand)]
pub enum ServicesCommand {
    Create(Box<ServiceCreateArgs>),
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
pub enum ServiceBindingsCommand {
    Create {
        #[arg(long, default_value = ".")]
        repo: Utf8PathBuf,
        #[arg(long)]
        scope: String,
        #[arg(long)]
        rule_id: String,
        #[arg(long)]
        service_id: String,
        #[arg(long)]
        binding_type: String,
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
