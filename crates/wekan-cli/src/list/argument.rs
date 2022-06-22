use crate::{
    command::{ArgumentRequester, ArtifactName, CommonCommandRequester, SubCommandValidator},
    error::kind::{CliError, Error, Transform},
    subcommand::CommonCommand as Command,
};
use clap::Args as ClapArgs;
use wekan_cli_derive::{CommonSubcommands, WekanArgs};
#[derive(ClapArgs, Debug, Clone, WekanArgs, CommonSubcommands)]
#[clap(version = "0.1.0", about = "Manage lists")]
pub struct Args {
    #[clap(short, long, help = "List name")]
    pub name: Option<String>,
    #[clap(short = 'b', long, help = "Board name")]
    pub board: String,
    #[clap(subcommand)]
    pub command: Option<Command>,
}

#[cfg(test)]
impl Args {
    pub fn mock(name: Option<String>, board: String, command: Option<Command>) -> Self {
        Args {
            name,
            board,
            command,
        }
    }
}
