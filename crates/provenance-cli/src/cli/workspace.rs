use crate::output::OutputFormat;
use camino::Utf8PathBuf;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum DocsCommand {
    Check {
        #[arg(long, default_value = ".")]
        repo: Utf8PathBuf,
        #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
        format: OutputFormat,
    },
    Serve {
        #[arg(long, default_value = ".")]
        repo: Utf8PathBuf,
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        #[arg(long, default_value_t = 5174)]
        port: u16,
    },
}

#[derive(Subcommand)]
pub enum WikiCommand {
    Build {
        #[arg(long, default_value = ".")]
        repo: Utf8PathBuf,
        #[arg(long, default_value = "default")]
        scope: String,
        /// Defaults to `.provenance/wiki`, which is added to `.gitignore`
        /// automatically. Pass an explicit path to write elsewhere instead
        /// (`.gitignore` is left untouched in that case).
        #[arg(long)]
        out: Option<Utf8PathBuf>,
        #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
        format: OutputFormat,
    },
    Serve {
        #[arg(long, default_value = ".")]
        repo: Utf8PathBuf,
        #[arg(long, default_value = "default")]
        scope: String,
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        #[arg(long, default_value_t = 5175)]
        port: u16,
    },
}

#[derive(Subcommand)]
pub enum SkillsCommand {
    List {
        #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
        format: OutputFormat,
    },
    Show {
        name: String,
    },
    Install {
        #[arg(long)]
        global: bool,
        #[arg(long)]
        copy: bool,
        #[arg(long)]
        force: bool,
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
