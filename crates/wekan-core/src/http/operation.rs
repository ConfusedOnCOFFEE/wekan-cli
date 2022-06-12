use crate::{config::ArtifactApi, error::kind::Error};
use async_trait::async_trait;
use log::debug;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    clone::Clone,
    fmt::Debug,
    marker::{Send, Sync},
};
use wekan_common::artifact::common::{AType, Artifact, Base};

use super::{artifact::ArtifactClient, util::SatisfyType};

#[cfg(feature = "store")]
use crate::{
    client::Client,
    config::{ConfigRequester, UserConfig},
    persistence::store::Store,
};
#[cfg(feature = "store")]
use log::trace;
#[cfg(feature = "store")]
use serde::Deserialize;
#[cfg(feature = "store")]
use wekan_common::artifact::common::StoreTrait;
#[cfg(feature = "store")]
impl Store for UserConfig {}
#[cfg(feature = "store")]
#[async_trait]
pub trait Artifacts: Operation + ConfigRequester<UserConfig> {
    async fn get_all(&mut self, t: AType) -> Result<Vec<Artifact>, Error> {
        let r = self.get_artifacts_url().to_owned();
        debug!("get_all {:?}", r);
        match self.get_vec::<Artifact>(&r).await {
            Ok(mut v) => {
                v.satisfy(t);
                #[cfg(feature = "store")]
                let c = self.get_config().clone();
                #[cfg(feature = "store")]
                let prepare_for_store = self.get_base_id();
                #[cfg(feature = "store")]
                <Client as Artifacts>::update_store(c, v.clone(), &prepare_for_store).await;
                Ok(v)
            }
            Err(e) => Err(e),
        }
    }
    async fn get_one<T: Base + StoreTrait + DeserializeOwned + 'static>(
        &mut self,
        id: &str,
    ) -> Result<T, Error> {
        let url = self.get_artifact_url(id);
        debug!("get_one {:?}", url);
        match self.get_details::<T>(&url).await {
            Ok(mut r) => {
                trace!("{:?}", r);
                r.set_id(id);
                #[cfg(feature = "store")]
                let c = self.get_config().clone();
                #[cfg(feature = "store")]
                let prepare_for_store = self.get_base_id() + id;
                #[cfg(feature = "store")]
                <Client as Artifacts>::update_store(c, r.clone(), &prepare_for_store).await;
                Ok(r)
            }
            Err(e) => Err(e),
        }
    }

    #[cfg(feature = "store")]
    async fn update_store<'de, T: StoreTrait + Deserialize<'de>>(
        config: UserConfig,
        body: T,
        id: &str,
    ) {
        let config = UserConfig {
            address: config.address.clone(),
            usertoken: config.usertoken.clone(),
            context: config.context.clone(),
        };
        <UserConfig as Store>::write_into_context::<T>(&config, body.to_owned(), id).await
    }
}
#[cfg(not(feature = "store"))]
#[async_trait]
pub trait Artifacts: Operation {
    async fn get_all(&mut self, t: AType) -> Result<Vec<Artifact>, Error> {
        let r = self.get_artifacts_url().to_owned();
        debug!("get_all {:?}", r);
        match self.get_vec(&r).await {
            Ok(mut v) => {
                v.satisfy(t);
                debug!("Response: {:?}", v);
                Ok(v)
            }
            Err(e) => Err(e),
        }
    }
    async fn get_one<T: Base + std::fmt::Debug + DeserializeOwned + 'static>(
        &mut self,
        id: &str,
    ) -> Result<T, Error> {
        let url = self.get_artifact_url(id);
        debug!("get_one {:?}", url);
        match self.get_details::<T>(&url).await {
            Ok(mut r) => {
                r.set_id(id);
                Ok(r)
            }
            Err(e) => Err(e),
        }
    }
}

#[async_trait]
pub trait Operation: ArtifactApi + ArtifactClient {
    async fn create<
        U: Send + Clone + Sync + Debug + Serialize,
        T: Send + Debug + DeserializeOwned + 'static,
    >(
        &mut self,
        body: &U,
    ) -> Result<T, Error> {
        let r = self.get_artifacts_url().to_owned();
        debug!("create {:?}", r);
        self.post_artifact(&r, body).await
    }
    async fn delete<T: Send + Debug + DeserializeOwned + 'static>(
        &mut self,
        id: &str,
    ) -> Result<T, Error> {
        let url = self.get_artifact_url(id);
        debug!("delete {:?}", url);
        self.delete_artifact(&url).await
    }
    async fn put<
        U: Send + Sync + Debug + Serialize + Clone,
        T: Send + Debug + DeserializeOwned + 'static,
    >(
        &mut self,
        id: &str,
        body: &U,
    ) -> Result<T, Error> {
        let url = self.get_artifact_url(id);
        debug!("put {:?}", url);
        self.put_artifact(&url, body).await
    }
}
