use crate::cli::DocsCommand;

pub(super) async fn handle(command: DocsCommand) -> anyhow::Result<()> {
    match command {
        DocsCommand::Check { repo, format } => crate::docs::check(&repo, format)?,
        DocsCommand::Serve { repo, host, port } => crate::docs::serve(repo, host, port).await?,
    }
    Ok(())
}
