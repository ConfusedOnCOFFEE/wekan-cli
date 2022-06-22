use crate::{
    command::{ArgumentRequester, ArtifactName, CommonCommandRequester, SubCommandValidator},
    error::kind::{CliError, Error, Transform},
    subcommand::CommonCommand as Command,
};
use clap::Args as ClapArgs;
use wekan_cli_derive::{CommonSubcommands, WekanArgs};

/// Board commands
#[derive(ClapArgs, Debug, Clone, WekanArgs, CommonSubcommands)]
#[clap(version = "0.1.0", about = "Manage boards")]
pub struct Args {
    #[clap(short, long, help = "Board name")]
    pub name: Option<String>,
    /// ls: Option<String>,
    /// Subcommands for board config, show swimlanes, lists.
    #[clap(subcommand)]
    pub command: Option<Command>,
}

#[cfg(test)]
impl Args {
    pub fn mock(name: Option<String>, command: Option<Command>) -> Self {
        Args { name, command }
    }
}
