use crate::subcommand::CommonCommand;
use clap::Args as ClapArgs;

#[derive(ClapArgs, Debug, Clone)]
#[clap(version = "0.1.0", about = "Manage lists")]
pub struct Args {
    /// List name
    pub name: Option<String>,
    #[clap(short = 'b', long, help = "Board name")]
    pub board: String,
    #[clap(short, long, parse(from_flag), help = "Show the details of the object")]
    raw: bool,
    #[clap(subcommand)]
    pub command: Option<CommonCommand>,
}
