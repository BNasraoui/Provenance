use crate::cli::workspace::WikiCommand;

pub(super) async fn handle(command: WikiCommand) -> anyhow::Result<()> {
    match command {
        WikiCommand::Build {
            repo,
            scope,
            out,
            coverage,
            format,
        } => crate::wiki::site::build(repo, scope, out, coverage.as_deref(), format)?,
        WikiCommand::Serve {
            repo,
            scope,
            coverage,
            host,
            port,
        } => crate::wiki::site::serve(repo, scope, coverage.as_deref(), host, port).await?,
    }
    Ok(())
}
