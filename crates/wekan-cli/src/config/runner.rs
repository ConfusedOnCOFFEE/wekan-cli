use crate::{command::BaseCommand, error::Error, result::WekanResult};

use super::{
    argument::{Args, Command as ConfigCommand},
    credentials::Authenticate,
};
#[cfg(feature = "store")]
#[cfg(feature = "store")]
use crate::config::{context::Context, credentials::ClearConfig};
#[cfg(feature = "store")]
use crate::error::{CliError, Transform};
use async_trait::async_trait;
#[cfg(feature = "store")]
use clap::Args as ClapArgs;
#[cfg(feature = "store")]
use log::{info, trace};
use wekan_core::client::LoginClient as Client;

#[cfg(feature = "store")]
use wekan_core::{config::UserConfig, persistence::store::Butler};

#[derive(Debug, Clone)]
pub struct Runner {
    pub args: Args,
    pub client: Client,
}

impl Runner {
    pub async fn use_subcommand(&mut self) -> Result<WekanResult, Error> {
        match self.args.command.to_owned() {
            ConfigCommand::SetCredentials(a) => self.client.run_login(&a).await,
            #[cfg(feature = "store")]
            ConfigCommand::DeleteCredentials(_a) => self.client.run_logout().await,
            #[cfg(feature = "store")]
            ConfigCommand::SetContext(a) => self.client.set(&a).await,
            #[cfg(feature = "store")]
            ConfigCommand::DeleteContext(a) => self.client.delete(&a).await,
            #[cfg(feature = "store")]
            ConfigCommand::UseContext(a) => self.client.r#use(&a).await,
            #[cfg(feature = "store")]
            ConfigCommand::Remove(rm) => self.remove_config(&rm).await,
        }
    }
    #[cfg(feature = "store")]
    async fn remove_config(&mut self, rm_args: &RemoveConfig) -> Result<WekanResult, Error> {
        match &rm_args.context {
            Some(context) => {
                let path_to_be_deleted = <UserConfig as Butler>::get_default_path() + context;
                info!("{:?}", path_to_be_deleted);
                match tokio::fs::remove_dir_all(path_to_be_deleted).await {
                    Ok(_v) => WekanResult::new_msg("Context and config successfully deleted").ok(),
                    Err(e) => {
                        trace!("{:?}", e);
                        CliError::new_msg("Context removal didn't work").err()
                    }
                }
            }
            None => self.remove_dir_all().await,
        }
    }
    #[cfg(feature = "store")]
    async fn remove_dir_all(&self) -> Result<WekanResult, Error> {
        let path_to_be_deleted = <UserConfig as Butler>::get_default_path();
        info!("{:?}", path_to_be_deleted);
        match tokio::fs::remove_dir_all(path_to_be_deleted).await {
            Ok(_v) => WekanResult::new_msg("wekan-cli config removed").ok(),
            Err(e) => {
                trace!("{:?}", e);
                CliError::new_msg("Path couldn't be deleted. Do it manually please!").err()
            }
        }
    }
}
#[async_trait]
impl BaseCommand<Args, Client> for Runner {
    fn new(args: Args, client: Client) -> Self {
        Self { args, client }
    }
}

#[cfg(feature = "store")]
#[derive(ClapArgs, Debug, Clone)]
#[clap(version = "0.1.0", about = "Remove config or context")]
pub struct RemoveConfig {
    #[clap(
        requires = "remove_complete",
        validator = only_config_allowed,
    )]
    pub please: String,
    #[clap(
        short = 'f',
        long,
        group = "remove_complete",
        parse(from_flag),
        help = "Confirm deletion"
    )]
    pub confirm: bool,
    #[clap(
        short = 'c',
        requires = "remove_context",
        long,
        help = "Select context to remove"
    )]
    pub context: Option<String>,
    #[clap(
        short = 'y',
        long,
        parse(from_flag),
        group = "remove_context",
        help = "Confirm context selection"
    )]
    pub context_confirm: bool,
}
#[cfg(feature = "store")]
fn only_config_allowed(s: &str) -> Result<(), String> {
    if s.contains("please") && s.len() == 6 {
        Ok(())
    } else {
        Err(String::from("If you want to remove, say please."))
    }
}
