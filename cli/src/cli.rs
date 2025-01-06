use clap::{Parser, Subcommand};
use crate::explorer_cli::ExplorerCommand;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new Bonsol project
    Init {
        /// Name of the project
        name: String,
        /// Optional directory for the project
        #[arg(short, long)]
        dir: Option<String>,
    },
    /// Explorer commands for tracking execution requests
    Explorer {
        #[command(subcommand)]
        command: ExplorerCommand,
    },
} 
