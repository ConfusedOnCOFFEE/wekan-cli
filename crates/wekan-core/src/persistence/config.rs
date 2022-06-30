use crate::{
    config::{ConfigRequester, UserConfig},
    error::Error,
};
use async_trait::async_trait;
#[cfg(not(test))]
use log::{info, trace};
use serde::{Deserialize, Serialize};
#[cfg(not(test))]
use std::fs;
#[cfg(not(test))]
use tokio::fs::File;
#[cfg(not(test))]
use tokio::io::AsyncWriteExt;

#[cfg(test)]
use crate::config::MandatoryConfig;

#[async_trait]
impl Butler for UserConfig {}
#[async_trait]
pub trait PersistenceConfig {
    async fn write_into_config<'de, T: Send + Deserialize<'de> + Serialize>(
        &self,
        partial_config: T,
    );
    async fn write_config(&self, config: UserConfig);
    async fn read_config(&self) -> Result<UserConfig, Error>;
}
#[cfg(not(test))]
#[async_trait]
impl PersistenceConfig for UserConfig {
    async fn write_into_config<'de, T: Send + Deserialize<'de> + Serialize>(
        &self,
        partial_config: T,
    ) {
        self.write("/config".to_string(), partial_config).await;
    }

    async fn write_config(&self, config: UserConfig) {
        info!("/write_context");
        self.write_into_config(config).await;
    }

    async fn read_config(&self) -> Result<UserConfig, Error> {
        let config_path = self.get_path();
        info!("Read file: {}", config_path);
        match tokio::fs::read(config_path.to_owned() + "/config").await {
            Ok(v) => match serde_yaml::from_slice::<UserConfig>(&v) {
                Ok(c) => {
                    trace!("Success: {:?}", c);
                    Ok(c)
                }
                Err(e) => Err(Error::Yaml(e)),
            },
            Err(e) => Err(Error::Io(e)),
        }
    }
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

#[cfg(test)]
#[async_trait]
impl PersistenceConfig for UserConfig {
    async fn write_into_config<'de, T: Send + Deserialize<'de> + Serialize>(
        &self,
        partial_config: T,
    ) {
        self.write("/config".to_string(), partial_config).await;
    }

    async fn write_config(&self, config: UserConfig) {
        self.write_into_config(config).await;
    }

    async fn read_config(&self) -> Result<UserConfig, Error> {
        Ok(UserConfig::new())
    }
}

#[async_trait]
pub trait FileWriter {
    async fn write<'de, T: Send + Deserialize<'de> + Serialize>(&self, path: String, artifact: T);
}

#[cfg(not(test))]
#[async_trait]
impl FileWriter for UserConfig {
    async fn write<'de, T: Send + Deserialize<'de> + Serialize>(&self, path: String, artifact: T) {
        let s: String = serde_yaml::to_string(&artifact).unwrap();
        let config_path = self.get_path();
        if !config_path.is_empty() {
            match fs::create_dir_all(&config_path) {
                Ok(_created) => {
                    let mut file = File::create(config_path.to_owned() + &path.to_owned())
                        .await
                        .unwrap();
                    file.write_all(s.as_bytes()).await.unwrap();
                }
                Err(_e) => panic!(
                    "Directory couldn't be created. Are you sure about the env WEKAN_CONFIG_PATH?"
                ),
            }
        }
    }
}
#[cfg(test)]
#[async_trait]
impl FileWriter for UserConfig {
    async fn write<'de, T: Send + Deserialize<'de> + Serialize>(
        &self,
        _path: String,
        _artifact: T,
    ) {
    }
}
