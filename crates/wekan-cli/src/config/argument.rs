#[cfg(feature = "store")]
use super::{
    context::{DeleteContext, SetContext, UseContext},
    runner::RemoveConfig,
};
use clap::{Args as ClapArgs, Subcommand};

use super::credentials::{DeleteCredentials, SetCredentials};

/// Config
#[derive(ClapArgs, Clone, Debug)]
#[clap(
    about = "CLI configuration",
    long_about = "Set, use and remove context or configs"
)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Command,
}

/// The following commands are available:
#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    #[clap(name = "set-credentials")]
    SetCredentials(SetCredentials),
    #[clap(name = "remove-credentials")]
    DeleteCredentials(DeleteCredentials),
    #[cfg(feature = "store")]
    #[clap(name = "use-context")]
    UseContext(UseContext),
    #[cfg(feature = "store")]
    #[clap(name = "set-context")]
    SetContext(SetContext),
    #[cfg(feature = "store")]
    #[clap(name = "remove-context")]
    DeleteContext(DeleteContext),
    #[cfg(feature = "store")]
    #[clap(name = "remove")]
    Remove(RemoveConfig),
}
