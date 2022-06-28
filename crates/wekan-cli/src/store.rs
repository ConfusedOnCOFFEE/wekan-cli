use crate::{
    error::{Error, StoreError},
    resolver::Query,
};
use async_trait::async_trait;
use log::{debug, info, trace};
use wekan_common::artifact::common::{AType, Artifact, IdReturner, SortedArtifact};
use wekan_core::persistence::store::Entry;

#[cfg(test)]
use chrono::prelude::*;
#[cfg(test)]
use wekan_common::artifact::tests::MockDetails;
#[cfg(not(test))]
use wekan_core::persistence::store::Butler;

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
impl<'a> Store for Query<'a> {
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
        info!("lookup_artifacts");
        trace!("{:?}", artifact);
        match self.stock_up(&artifact).await {
            Ok(a) => Ok(a),
            Err(_e) => Err(Error::Store(StoreError { found: false })),
        }
    }
    async fn lookup_id(&self, artifact: &Artifact) -> Result<String, Error> {
        info!("lookup_id");
        trace!("{:?}", artifact);
        let identifier = artifact.get_type().to_string() + "_" + &artifact.get_id();
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
            _ => {
                config_path.to_owned() + &artifact.get_type().to_string() + "_" + &artifact.get_id()
            }
        };
        trace!("Load from: {:?}", to_load_from);
        match tokio::fs::read(to_load_from).await {
            Ok(v) => match serde_yaml::from_slice::<Entry<Vec<Artifact>>>(&v) {
                Ok(v) => {
                    trace!("Success: {:?}", v);
                    Ok(v)
                }
                Err(e) => Err(Error::Yaml(e)),
            },
            Err(e) => Err(Error::Io(e)),
        }
    }

    #[cfg(test)]
    async fn stock_up(&self, artifact: &Artifact) -> Result<Entry<Vec<Artifact>>, Error> {
        let id_prefix = String::from("store-fake-");
        let id_suffix = String::from("-id-");
        let title_prefix = String::from("store-fake-");
        let title_suffix = String::from("-title-");
        Ok(Entry {
            parent: artifact.get_id(),
            age: Utc::now().to_string(),
            payload: vec![
                Artifact::mock(
                    &(id_prefix.to_owned() + &artifact.get_type().to_string() + &id_suffix + "1"),
                    &(title_prefix.to_owned()
                        + &artifact.get_type().to_string()
                        + &title_suffix
                        + "1"),
                    &artifact.get_type().to_string(),
                ),
                Artifact::mock(
                    &(id_prefix + &artifact.get_type().to_string() + &id_suffix + "2"),
                    &(title_prefix + &artifact.get_type().to_string() + &title_suffix + "2"),
                    &artifact.get_type().to_string(),
                ),
            ],
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
            Ok(v) => match serde_yaml::from_slice::<Artifact>(&v) {
                Ok(c) => {
                    trace!("Success: {:?}", c);
                    if unqiue_identifier == (c._id.to_owned() + &c.title) {
                        debug!("Loaded artifact match");
                        Ok(c._id)
                    } else {
                        Err(Error::Store(StoreError { found: false }))
                    }
                }
                Err(e) => Err(Error::Yaml(e)),
            },
            Err(e) => Err(Error::Io(e)),
        }
    }
}
