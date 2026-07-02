use serde::Serialize;

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum OutputFormat {
    Json,
    Jsonl,
    Markdown,
    Table,
    Toon,
}

pub fn print<T: Serialize>(format: OutputFormat, value: &T) -> anyhow::Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(value)?),
        OutputFormat::Jsonl => println!("{}", serde_json::to_string(value)?),
        OutputFormat::Markdown | OutputFormat::Table | OutputFormat::Toon => {
            println!("{}", serde_json::to_string_pretty(value)?);
        }
    }
    Ok(())
}
