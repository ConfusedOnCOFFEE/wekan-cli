use async_trait::async_trait;
use crate::{
    error::kind::{Error, StoreError},
    resolver::Query,
};
use wekan_common::artifact::common::{
    AType,
    Artifact,
    IdReturner,
    // Base,
    SortedArtifact,
};
use wekan_core::persistence::store::Entry;
use log::{debug, info, trace};

#[cfg(not(test))]
use wekan_core::persistence::store::Butler;
#[cfg(test)]
use chrono::{prelude::*};
#[async_trait]
pub trait Store {
    async fn lookup_id(&self, artifact: &Artifact) -> Result<String, Error>;
    async fn approve_id(&self, id: &str) -> Result<String, Error>;
    async fn lookup_artifacts(
        &self,
        artifact_variant: AType,
        id: &str,
    ) -> Result<Entry<Vec<Artifact>>, Error>;
    async fn stock_up(&self, artifact: &Artifact) -> Result<Entry<Vec<Artifact>>, Error>;
}

#[async_trait]
impl Store for Query {
    async fn lookup_artifacts(
        &self,
        artifact_variant: AType,
        id: &str,
    ) -> Result<Entry<Vec<Artifact>>, Error> {
        let artifact = Artifact {
            _id: id.to_string(),
            title: String::new(),
            r#type: artifact_variant,
        };
        info!("request_artifacts");
        trace!("{:?}", artifact);
        match self.stock_up(&artifact).await {
            Ok(a) => Ok(a),
            Err(_e) => Err(Error::Store(StoreError { found: false })),
        }
    }
    async fn lookup_id(&self, artifact: &Artifact) -> Result<String, Error> {
        info!("request_artifact");
        trace!("{:?}", artifact);
        let identifier = artifact.get_type().to_string() + &artifact.get_id();
        trace!("Identifier: {:?}", identifier);
        match self.approve_id(&identifier).await {
            Ok(a) => Ok(a),
            Err(_e) => Err(Error::Store(StoreError { found: false })),
        }
    }

    #[cfg(not(test))]
    async fn stock_up(&self, artifact: &Artifact) -> Result<Entry<Vec<Artifact>>, Error> {
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

    #[cfg(test)]
    async fn stock_up(&self, artifact: &Artifact) -> Result<Entry<Vec<Artifact>>, Error> {
        Ok(Entry  {
            parent: artifact.get_id(),
            age: Utc::now().to_string(),
            payload: vec![
                Artifact::new("store-fake-id-1", "store-fake-title", artifact.get_type().clone()),
                Artifact::new("store-fake-id-2", "store-fake-title2", artifact.get_type()),
            ]
        })
    }
    async fn approve_id(&self, unqiue_identifier: &str) -> Result<String, Error> {
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
