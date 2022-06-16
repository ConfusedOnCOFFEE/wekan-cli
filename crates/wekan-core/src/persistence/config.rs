use super::store::Butler;
use crate::{
    config::{MandatoryConfig, UserConfig},
    error::kind::Error,
};
use async_trait::async_trait;
use log::debug;
#[cfg(not(test))]
use log::trace;
use serde::{Deserialize, Serialize};
#[cfg(not(test))]
use std::fs;
#[cfg(not(test))]
use tokio::fs::File;
#[cfg(not(test))]
use tokio::io::AsyncWriteExt;

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
        debug!("/write_context");
        self.write_into_config(config).await;
    }

    async fn read_config(&self) -> Result<UserConfig, Error> {
        debug!("read_config");
        let config_path = self.get_path();
        debug!("read from file: {}", config_path);
        match tokio::fs::read(config_path.to_owned() + "/config").await {
            Ok(v) => match String::from_utf8_lossy(&v).parse::<String>() {
                Ok(s) => {
                    trace!("{:?}", s);
                    match serde_yaml::from_slice::<UserConfig>(&v) {
                        Ok(c) => {
                            trace!("Read succesffully: {:?}", c);
                            Ok(c)
                        }
                        Err(e) => Err(Error::Yaml(e)),
                    }
                }
                Err(_e) => Ok(UserConfig::new()),
            },
            Err(e) => Err(Error::Io(e)),
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
        debug!("/write_context");
        self.write_into_config(config).await;
    }

    async fn read_config(&self) -> Result<UserConfig, Error> {
        debug!("read_config");
        let config_path = self.get_path();
        debug!("read from file: {}", config_path);
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
        debug!("write to file: {}{}", config_path, path);
        if !config_path.is_empty() {
            match fs::create_dir_all(config_path.to_owned()) {
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
