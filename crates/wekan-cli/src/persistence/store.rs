use async_trait::async_trait;
//use clap::YamlLoader;
// use tokio::{fs::File, io::{AsyncReadExt, AsyncWriteExt}};
use crate::{
    resolver::Query,
    error::kind::{Error, StoreError},
};
use wekan_common::artifact::common::{AType, Artifact, SortedArtifact, Base};
use wekan_core::persistence::store::{Butler, Entry};
// use std::fs;
use log::{debug, info, trace};
// use serde::{Deserialize, Serialize};

#[async_trait]
pub trait Store {
    async fn request_artifact(&self, artifact: &Artifact) -> Result<String, Error>;
    async fn request_artifacts(
        &self,
        artifact_variant: AType,
        id: &str
    ) -> Result<Entry<Vec<Artifact>>, Error>;
    async fn load_artifacts(&self, artifact: &Artifact) -> Result<Entry<Vec<Artifact>>, Error>;
    async fn load_last_used_artifact(&self, id: &str) -> Result<String, Error>;
}

#[async_trait]
impl Store for Query {
    async fn request_artifacts(
        &self,
        artifact_variant: AType,
        id: &str,
    ) -> Result<Entry<Vec<Artifact>>, Error> {
        let artifact = Artifact {
            _id: id.to_string(),
            title: String::new(),
            r#type: artifact_variant
        };
        info!("request_artifacts");
        trace!("{:?}", artifact);
        match self.load_artifacts(&artifact).await {
            Ok(a) => Ok(a),
            Err(_e) => Err(Error::Store(StoreError { found: false })),
        }
    }
    async fn request_artifact(&self, artifact: &Artifact) -> Result<String, Error> {
        info!("request_artifact");
        trace!("{:?}", artifact);
        let identifier = artifact.get_type().to_string() + &artifact.get_id();
        trace!("Identifier: {:?}", identifier);
        match self.load_last_used_artifact(&identifier).await {
            Ok(a) => Ok(a),
            Err(_e) => Err(Error::Store(StoreError { found: false })),
        }
    }

    async fn load_artifacts(&self, artifact: &Artifact) -> Result<Entry<Vec<Artifact>>, Error> {
        let config_path = self.config.get_path();
        let to_load_from = match artifact.get_type() {
            AType::Board => config_path.to_owned() + &artifact.get_type().to_string() + "s",
            _ => config_path.to_owned() + &artifact.get_type().to_string() + &artifact.get_id(),
        };
        trace!("Load artifact from: {:?}", to_load_from);
        match tokio::fs::read(to_load_from).await {
            Ok(v) => match String::from_utf8_lossy(&v).parse::<String>() {
                Ok(s) => {
                    trace!("{:?}", s);
                    match serde_yaml::from_slice::<Entry<Vec<Artifact>>>(&v) {
                        Ok(v) => {
                            trace!("Read succesffully: {:?}", v);
                            Ok(v)
                        }
                        Err(e) => Err(Error::Yaml(e)),
                    }
                }
                Err(_e) => Err(Error::Store(StoreError { found: false })),
            },
            Err(e) => Err(Error::Io(e)),
        }
    }
    async fn load_last_used_artifact(&self, unqiue_identifier: &str) -> Result<String, Error> {
        let config_path = match std::env::var("WEKAN_CLI_CONFIG_PATH") {
            Ok(config_path_env) => config_path_env,
            Err(_e) => {
                println!("Setting up config not possible. No WEKAN_CLI_CONFIG_PATH set.!");
                String::new()
            }
        };
        trace!("Identifier: {:?}", unqiue_identifier);
        match tokio::fs::read(config_path.to_owned() + unqiue_identifier).await {
            Ok(v) => match String::from_utf8_lossy(&v).parse::<String>() {
                Ok(s) => {
                    trace!("{:?}", s);
                    match serde_yaml::from_slice::<Artifact>(&v) {
                        Ok(c) => {
                            trace!("Read succesffully: {:?}", c);
                            if unqiue_identifier == (c._id.to_owned() + &c.title) {
                                debug!("It is a match!");
                                Ok(c._id)
                            } else {
                                Err(Error::Store(StoreError { found: false }))
                            }
                        }
                        Err(e) => Err(Error::Yaml(e)),
                    }
                }
                Err(_e) => Err(Error::Store(StoreError { found: false })),
            },
            Err(e) => Err(Error::Io(e)),
        }
    }
}
