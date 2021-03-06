use async_trait::async_trait;
use clap::Args as ClapArgs;
use log::{debug, info, trace};
use wekan_core::persistence::config::Butler;
use wekan_core::{client::LoginClient as Client, config::UserConfig};

use crate::{
    error::{CliError, Error, Transform},
    result::WekanResult,
};

#[derive(ClapArgs, Debug, Clone)]
#[clap(
    about = "Set context",
    long_about = "Set a context to isolate store and login"
)]
pub struct SetContext {
    /// Context
    pub name: String,
}

#[cfg(feature = "store")]
#[derive(ClapArgs, Debug, Clone)]
#[clap(
    about = "Remove context",
    long_about = "Remove a context. This clears store and login"
)]
pub struct DeleteContext {
    // Context
    pub name: String,
}

#[derive(ClapArgs, Debug, Clone)]
#[clap(about = "Use context", long_about = "Switch between contexts")]
pub struct UseContext {
    /// Context
    pub name: String,
}

#[async_trait]
pub trait Context {
    async fn set(&self, set_context: &SetContext) -> Result<WekanResult, Error>;
    async fn delete(&self, delete_context: &DeleteContext) -> Result<WekanResult, Error>;
    async fn r#use(&self, use_context: &UseContext) -> Result<WekanResult, Error>;
    async fn default(&self, name: &str) -> Result<WekanResult, Error>;
}
#[async_trait]
impl Context for Client {
    async fn set(&self, set_context: &SetContext) -> Result<WekanResult, Error> {
        info!("set context");
        let new_context =
            <UserConfig as Butler>::get_default_path() + &set_context.name + "/config";
        let current_context = <UserConfig as Butler>::get_default_path() + "config";
        debug!("NC: {}", new_context);
        debug!("CC: {}", current_context);
        match tokio::fs::create_dir_all(
            <UserConfig as Butler>::get_default_path().to_owned() + &set_context.name,
        )
        .await
        {
            Ok(_v) => match tokio::fs::copy(current_context, new_context).await {
                Ok(_v) => WekanResult::new_msg("Context saved").ok(),
                Err(e) => {
                    trace!("{:?}", e);
                    CliError::new_msg("New context couldn't be created").err()
                }
            },
            Err(e) => {
                trace!("{:?}", e);
                CliError::new_msg("New context couldn't be created").err()
            }
        }
    }
    async fn delete(&self, delete_context: &DeleteContext) -> Result<WekanResult, Error> {
        info!("delete context");
        let config_path =
            <UserConfig as Butler>::get_default_path() + &delete_context.name + "/config";
        debug!("NC: {}", config_path);
        match tokio::fs::remove_file(config_path).await {
            Ok(_ok) => WekanResult::new_workflow("Successfully deleted", "config set-context").ok(),
            Err(_e) => WekanResult::new_msg("Context couldn't be deleted. It doesn't exist.").ok(),
        }
    }
    async fn r#use(&self, use_context: &UseContext) -> Result<WekanResult, Error> {
        debug!("use context");
        self.default(&use_context.name).await
    }
    async fn default(&self, name: &str) -> Result<WekanResult, Error> {
        debug!("default");
        let new_context = <UserConfig as Butler>::get_default_path() + name + "/config";
        let context_to_be = <UserConfig as Butler>::get_default_path() + "/config";
        debug!("NC: {}", new_context);
        debug!("CC: {}", context_to_be);
        match tokio::fs::copy(new_context, context_to_be).await {
            Ok(_v) => WekanResult::new_msg("Using specified context").ok(),
            Err(_e) => CliError::new_msg("New context doesn't exist").err(),
        }
    }
}
