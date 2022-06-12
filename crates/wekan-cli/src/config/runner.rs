use crate::{
    command::{BaseCommand, RootCommand},
    display::CliDisplay,
    error::kind::{CliError, Error, Transform},
    result::kind::WekanResult,
};

use super::{
    argument::{Args, Command as ConfigCommand},
    credentials::Authenticate,
};
use async_trait::async_trait;
#[cfg(feature = "store")]
use log::{info, trace};
use wekan_core::client::LoginClient as Client;

#[cfg(feature = "store")]
use crate::config::{context::Context, credentials::ClearConfig};
#[cfg(feature = "store")]
use clap::Args as ClapArgs;

#[cfg(feature = "store")]
use wekan_core::{config::UserConfig, persistence::store::Butler};

#[derive(Debug, Clone)]
pub struct Runner {
    pub args: Args,
    pub client: Client,
}

#[cfg(feature = "store")]
impl Runner {
    async fn remove_config(&mut self, rm_args: &RemoveConfig) -> Result<WekanResult, Error> {
        match &rm_args.context {
            Some(context) => {
                let path_to_be_deleted = <UserConfig as Butler>::get_default_path() + &context;
                info!("{:?}", path_to_be_deleted);
                match tokio::fs::remove_dir_all(path_to_be_deleted).await {
                    Ok(_v) => WekanResult::new_msg("Context and store deleted.").ok(),
                    Err(e) => {
                        trace!("{:?}", e);
                        CliError::new_msg("Context clearage didn't work").err()
                    }
                }
            }
            None => self.remove_dir_all().await,
        }
    }

    async fn remove_dir_all(&self) -> Result<WekanResult, Error> {
        let path_to_be_deleted = <UserConfig as Butler>::get_default_path();
        info!("{:?}", path_to_be_deleted);
        match tokio::fs::remove_dir_all(path_to_be_deleted).await {
            Ok(_v) => WekanResult::new_msg("Config completly cleared").ok(),
            Err(e) => {
                trace!("{:?}", e);
                CliError::new_msg("Path couldn't be deleted. Do it manually").err()
            }
        }
    }
}

impl CliDisplay for Runner {}
#[async_trait]
impl BaseCommand<Args, Client> for Runner {
    fn new(args: Args, client: Client) -> Self {
        Self { args, client }
    }
}

#[cfg(feature = "store")]
#[derive(ClapArgs, Debug, Clone)]
#[clap(version = "0.1.0", about = "Remove config or contexts.")]
pub struct RemoveConfig {
    #[clap(
        short = 'f',
        long,
        group = "clear_context_arg",
        parse(from_flag),
        help = "Confirm deletion"
    )]
    pub confirm: bool,
    #[clap(
        short = 'c',
        requires = "clear_context_arg",
        long,
        help = "Select context to remove"
    )]
    pub context: Option<String>,
    #[clap(
        short = 'y',
        long,
        parse(from_flag),
        group = "clear_context_arg",
        help = "Confirm context selection"
    )]
    pub context_confirm: bool,
}

#[async_trait]
impl RootCommand for Runner {
    async fn run(&mut self) -> Result<WekanResult, Error> {
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
    async fn use_rootcommand(&mut self, name: &str) -> Result<WekanResult, Error> {
        self.client.default(name).await
    }
    #[cfg(not(feature = "store"))]
    async fn use_rootcommand(&mut self, _name: &str) -> Result<WekanResult, Error> {
        CliError::new_msg(
            "Only subcommands are possible. Name field is unused in this build version.",
        )
        .err()
    }
}
