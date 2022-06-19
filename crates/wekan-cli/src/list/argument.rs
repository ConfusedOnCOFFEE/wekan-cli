use crate::subcommand::CommonCommand;
use clap::Args as ClapArgs;

#[derive(ClapArgs, Debug, Clone)]
#[clap(version = "0.1.0", about = "Manage lists")]
pub struct Args {
    /// List name
    pub name: Option<String>,
    #[clap(short = 'b', long, help = "Board name")]
    pub board: String,
    #[clap(subcommand)]
    pub command: Option<CommonCommand>,
}

#[cfg(test)]
impl Args {
    pub fn mock(name: Option<String>, board: String, command: Option<CommonCommand>) -> Self {
        Args {
            name,
            board,
            command,
        }
    }
}
