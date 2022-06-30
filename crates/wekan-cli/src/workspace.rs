use crate::error::Error;
use crate::runner::Runner;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use wekan_common::artifact::common::Artifact;
use wekan_core::persistence::config::Butler;

#[cfg(test)]
use chrono::prelude::*;
#[cfg(not(test))]
use std::fs;
#[cfg(not(test))]
use tokio::fs::File;
#[cfg(not(test))]
use tokio::io::AsyncWriteExt;
#[cfg(test)]
use wekan_common::artifact::{common::AType, tests::MockDetails};

#[async_trait]
pub trait Workspace {
    async fn setup(&self) -> Result<Vec<Artifact>, Error>;
    async fn write<'de, T: Send + Deserialize<'de> + Serialize>(&self, vec: Vec<T>);
}

#[async_trait]
impl Workspace for Runner {
    #[cfg(not(test))]
    async fn setup(&self) -> Result<Vec<Artifact>, Error> {
        let config_path = self.client.config.get_path() + "workspace";
        match tokio::fs::read(config_path).await {
            Ok(v) => match serde_yaml::from_slice::<Vec<Artifact>>(&v) {
                Ok(v) => Ok(v),
                Err(e) => Err(Error::Yaml(e)),
            },
            Err(e) => Err(Error::Io(e)),
        }
    }

    #[cfg(not(test))]
    async fn write<'de, T: Send + Deserialize<'de> + Serialize>(&self, vec: Vec<T>) {
        let s: String = serde_yaml::to_string(&vec).unwrap();
        let config_path = self.client.config.get_path();
        if !config_path.is_empty() {
            match fs::create_dir_all(&config_path) {
                Ok(_created) => {
                    let mut file = File::create(config_path.to_owned() + "workspace")
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

    #[cfg(test)]
    async fn write<'de, T: Send + Deserialize<'de> + Serialize>(&self, _vec: Vec<T>) {}

    #[cfg(test)]
    async fn setup(&self) -> Result<Vec<Artifact>, Error> {
        let id_prefix = "store-fake-";
        let id_suffix = "-id-";
        let title_prefix = "store-fake-";
        let title_suffix = "-title-";
        Ok(vec![
            Artifact::mock(
                &(id_prefix.to_owned() + &AType::Board.to_string() + &id_suffix + "1"),
                &(title_prefix.to_owned() + &AType::Board.to_string() + &title_suffix + "1"),
                &AType::Board.to_string(),
            ),
            Artifact::mock(
                &(id_prefix.to_owned() + &AType::List.to_string() + &id_suffix + "2"),
                &(title_prefix.to_owned() + &AType::List.to_string() + &title_suffix + "2"),
                &AType::List.to_string(),
            ),
            Artifact::mock(
                &(id_prefix.to_owned() + &AType::Card.to_string() + &id_suffix + "2"),
                &(title_prefix.to_owned() + &AType::Card.to_string() + &title_suffix + "2"),
                &AType::Card.to_string(),
            ),
            Artifact::mock(
                &(id_prefix.to_owned() + &AType::Checklist.to_string() + &id_suffix + "2"),
                &(title_prefix.to_owned() + &AType::Checklist.to_string() + &title_suffix + "2"),
                &AType::Checklist.to_string(),
            ),
        ])
    }
}
