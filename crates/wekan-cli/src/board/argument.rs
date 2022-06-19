use clap::Args as ClapArgs;
use std::path::PathBuf;

use crate::subcommand::CommonCommand;

/// Board commands
#[derive(ClapArgs, Debug, Clone)]
#[clap(version = "0.1.0", about = "Manage boards")]
pub struct Args {
    /// Specify the board to be updated.
    pub name: Option<String>,
    /// ls: Option<String>,
    /// Subcommands for board config, show swimlanes, lists.
    #[clap(subcommand)]
    pub command: Option<CommonCommand>,
    /// Specify the file to create/update a task.
    #[clap(short, long, parse(from_os_str), value_name = "FILE")]
    task_file: Option<PathBuf>,
}

#[cfg(test)]
impl Args {
    pub fn mock(
        name: Option<String>,
        command: Option<CommonCommand>,
        task_file: Option<PathBuf>,
    ) -> Self {
        Args {
            name,
            command,
            task_file,
        }
    }
}
