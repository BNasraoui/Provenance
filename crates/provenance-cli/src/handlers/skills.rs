use crate::cli::SkillsCommand;
use crate::output;
use crate::skills;

pub(super) fn handle(command: SkillsCommand) -> anyhow::Result<()> {
    match command {
        SkillsCommand::List { format } => output::print(format, &skills::list()?)?,
        SkillsCommand::Show { name } => print!("{}", skills::show(&name)?),
        SkillsCommand::Install {
            global,
            copy,
            force,
            format,
        } => output::print(format, &skills::install(global, force, copy)?)?,
    }
    Ok(())
}
