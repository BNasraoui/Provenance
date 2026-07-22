use crate::output::OutputFormat;
use camino::Utf8PathBuf;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum EdgesCommand {
    Create {
        #[arg(long, default_value = ".")]
        repo: Utf8PathBuf,
        #[arg(long)]
        scope: String,
        #[arg(long = "type")]
        edge_type: String,
        #[arg(long)]
        from_type: String,
        #[arg(long)]
        from_id: String,
        #[arg(long)]
        to_type: String,
        #[arg(long)]
        to_id: String,
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
    Delete {
        #[arg(long, default_value = ".")]
        repo: Utf8PathBuf,
        #[arg(long)]
        scope: String,
        #[arg(long)]
        id: String,
        #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
        format: OutputFormat,
    },
}

#[derive(Subcommand)]
pub enum GraphReferenceCommand {
    /// Issue an immutable reference after canonical graph state is committed.
    Issue {
        #[arg(long, default_value = ".")]
        repo: Utf8PathBuf,
        #[arg(long, default_value = "default")]
        scope: String,
        /// Git revision to pin. When omitted, clean relevant state at HEAD is required.
        #[arg(long)]
        commit: Option<String>,
        #[arg(long, requires = "correlation_key")]
        correlation_system: Option<String>,
        #[arg(long, requires = "correlation_system")]
        correlation_key: Option<String>,
    },
    /// Show metadata and graph-family counts from a pinned reference.
    Show {
        #[arg(long, default_value = ".")]
        repo: Utf8PathBuf,
        #[arg(long)]
        reference: Utf8PathBuf,
    },
    /// Verify identity and graph content at the pinned Git revision.
    Verify {
        #[arg(long, default_value = ".")]
        repo: Utf8PathBuf,
        #[arg(long)]
        reference: Utf8PathBuf,
    },
    /// Export the canonical graph reconstructed from the pinned Git revision.
    ExactExport {
        #[arg(long, default_value = ".")]
        repo: Utf8PathBuf,
        #[arg(long)]
        reference: Utf8PathBuf,
    },
}
