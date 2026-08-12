use crate::output::OutputFormat;
use camino::Utf8PathBuf;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum SdkCommand {
    /// Report engine compatibility and the resolved project root.
    Info {
        #[arg(long)]
        repo: Option<Utf8PathBuf>,
        #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
        format: OutputFormat,
    },
    /// Preview one desired-state reconciliation without writing it.
    Plan {
        #[arg(long)]
        repo: Option<Utf8PathBuf>,
        #[arg(long, default_value = "default")]
        scope: String,
        #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
        format: OutputFormat,
    },
    /// Reconcile one desired-state document read from stdin.
    Apply {
        #[arg(long)]
        repo: Option<Utf8PathBuf>,
        #[arg(long, default_value = "default")]
        scope: String,
        #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
        format: OutputFormat,
    },
    /// Start one callback-backed verification run described on stdin.
    BeginVerification {
        #[arg(long)]
        repo: Option<Utf8PathBuf>,
        #[arg(long, default_value = "default")]
        scope: String,
        #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
        format: OutputFormat,
    },
    /// Finish one callback-backed verification run described on stdin.
    CompleteVerification {
        #[arg(long)]
        repo: Option<Utf8PathBuf>,
        #[arg(long, default_value = "default")]
        scope: String,
        #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
        format: OutputFormat,
    },
    /// List cached callback-backed verification evidence.
    VerificationRuns {
        #[arg(long)]
        repo: Option<Utf8PathBuf>,
        #[arg(long, default_value = "default")]
        scope: String,
        #[arg(long)]
        rule: Option<String>,
        #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
        format: OutputFormat,
    },
    /// List durable typed verification relationships.
    VerificationBindings {
        #[arg(long)]
        repo: Option<Utf8PathBuf>,
        #[arg(long, default_value = "default")]
        scope: String,
        #[arg(long)]
        rule: Option<String>,
        #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
        format: OutputFormat,
    },
}
