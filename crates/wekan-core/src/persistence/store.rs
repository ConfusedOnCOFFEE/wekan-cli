#[cfg(not(test))]
use super::config::FileWriter;
use crate::config::{ConfigRequester, UserConfig};
use async_trait::async_trait;
#[cfg(not(test))]
use chrono::prelude::*;
#[cfg(not(test))]
use log::debug;
use serde::{Deserialize, Serialize};
use wekan_common::artifact::common::StoreTrait;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Entry<T> {
    pub age: String,
    pub parent: String,
    pub payload: T,
}

#[async_trait]
pub trait Butler: ConfigRequester<UserConfig> {
    fn get_path(&self) -> String {
        match self.get_config().context {
            Some(p) => Self::get_default_path() + &p + "/",
            None => Self::get_default_path(),
        }
    }

    fn get_default_path() -> String {
        match std::env::var("WEKAN_CLI_CONFIG_PATH") {
            Ok(config_path_env) => config_path_env,
            Err(_e) => {
                let home = std::env::var("HOME").unwrap();
                home + "/.config/wekan-cli/"
            }
        }
    }
}

#[cfg(not(test))]
#[async_trait]
pub trait Store: ConfigRequester<UserConfig> {
    async fn write_into_context<'de, T: StoreTrait + Deserialize<'de>>(
        &self,
        partial_context: T,
        id: &str,
    ) {
        let entry = {
            Entry {
                age: Utc::now().to_string(),
                parent: id.to_string(),
                payload: partial_context,
            }
        };
        debug!("Complete Entry: {:?}", entry);
        let mut path = entry.payload.get_type().to_string().to_owned();
        if entry.parent.to_string().is_empty() {
            path.push('s');
        } else {
            path.push_str(&entry.parent);
        }
        debug!("Path: {:?}", path);
        self.get_config().write(path.to_string(), entry).await;
    }
}

#[cfg(test)]
#[async_trait]
pub trait Store: ConfigRequester<UserConfig> {
    async fn write_into_context<'de, T: StoreTrait + Deserialize<'de>>(
        &self,
        _partial_context: T,
        _id: &str,
    ) {
    }
}
