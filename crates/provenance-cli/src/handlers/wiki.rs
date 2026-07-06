use crate::cli::WikiCommand;

pub(super) async fn handle(command: WikiCommand) -> anyhow::Result<()> {
    match command {
        WikiCommand::Build {
            repo,
            scope,
            out,
            format,
        } => crate::wiki::site::build(repo, scope, out, format)?,
        WikiCommand::Serve {
            repo,
            scope,
            host,
            port,
        } => crate::wiki::site::serve(repo, scope, host, port).await?,
    }
    Ok(())
}
