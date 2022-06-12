use clap::{Args as ClapArgs, Subcommand};

#[cfg(feature = "store")]
use super::{
    credentials::DeleteCredentials,
    context::{SetContext, DeleteContext, UseContext},
    runner::RemoveConfig
};

use super::credentials::SetCredentials;

/// Config
#[derive(ClapArgs, Clone, Debug)]
#[clap(version = "0.1.0", about = "CLI configuration.")]
pub struct Args {
    #[clap(subcommand)]
    pub command: Command,
}

/// The following commands are available:
#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    #[clap(name = "set-credentials")]
    SetCredentials(SetCredentials),
    #[cfg(feature = "store")]
    #[clap(name = "delete-credentials")]
    DeleteCredentials(DeleteCredentials),
    #[cfg(feature = "store")]
    #[clap(name = "use-context")]
    UseContext(UseContext),
    #[cfg(feature = "store")]
    #[clap(name = "set-context")]
    SetContext(SetContext),
    #[cfg(feature = "store")]
    #[clap(name = "delete-context")]
    DeleteContext(DeleteContext),
    #[cfg(feature = "store")]
    #[clap(name = "remove")]
    Remove(RemoveConfig)
}
