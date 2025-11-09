use crate::commands;
use crate::prelude::*;
use clap::{Args, Subcommand};

pub mod add;

#[derive(Debug, Clone, Subcommand)]
pub enum RepoCommand {
    /// Imports cheatsheets from a repo
    Add {
        /// A URI to a git repository containing .cheat files ("user/repo" will download cheats from github.com/user/repo)
        uri: String,
    },
}

#[derive(Debug, Clone, Args)]
pub struct Input {
    #[clap(subcommand)]
    pub cmd: RepoCommand,
}

impl Runnable for Input {
    fn run(&self) -> Result<()> {
        match &self.cmd {
            RepoCommand::Add { uri } => {
                add::main(uri.clone())
                    .with_context(|| format!("Failed to import cheatsheets from `{uri}`"))?;
                commands::core::main()
            }
        }
    }
}
