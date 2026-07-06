mod cli;
mod docs;
mod gitignore;
mod handlers;
mod output;
mod skills;
mod wiki;

use clap::Parser;
use cli::Cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let quiet = cli.quiet;
    handlers::dispatch(cli.command, quiet).await
}
