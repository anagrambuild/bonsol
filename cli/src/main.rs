mod cli;
mod init;
mod explorer;
mod explorer_cli;

use clap::Parser;
use cli::{Cli, Commands};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { name, dir } => {
            init::init_project(&name, dir)?;
        }
        Commands::Explorer { command } => {
            explorer_cli::handle_explorer_command(command)?;
        }
    }

    Ok(())
}
