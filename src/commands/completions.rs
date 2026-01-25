use crate::cli::CompletionsCommand;
use anyhow::Result;
use clap::CommandFactory;

pub async fn execute(cmd: CompletionsCommand) -> Result<()> {
    clap_complete::generate(
        cmd.shell,
        &mut crate::cli::Cli::command(),
        "rco",
        &mut std::io::stdout(),
    );
    Ok(())
}
