use super::util::SatisfyType;
#[cfg(feature = "store")]
use crate::{
    client::Client,
    config::{ConfigRequester, UserConfig},
    persistence::store::Store,
};
use crate::{config::ArtifactApi, error::Error, http::client::HttpClient};
use async_trait::async_trait;
use log::{info, trace};
#[cfg(feature = "store")]
use serde::Deserialize;
use std::marker::Send;
#[cfg(feature = "store")]
use wekan_common::artifact::common::StoreTrait;
use wekan_common::{
    artifact::common::{AType, Artifact, Base, DeserializeExt},
    http::artifact::{Deleted, RequestBody},
};
#[cfg(feature = "store")]
impl Store for UserConfig {}

#[cfg(feature = "store")]
#[async_trait]
pub trait Artifacts: Operation + ConfigRequester<UserConfig> {
    async fn get_all(&mut self, t: AType) -> Result<Vec<Artifact>, Error> {
        let r = self.get_artifacts_url().to_owned();
        info!("get_all {}", r);
        match self.get_vec::<Artifact>(&r).await {
            Ok(mut v) => {
                v.satisfy(t);
                #[cfg(feature = "store")]
                let c = self.get_config();
                #[cfg(feature = "store")]
                let prepare_for_store = self.get_base_id();
                #[cfg(feature = "store")]
                <Client as Artifacts>::update_store(c, v.clone(), &prepare_for_store).await;
                Ok(v)
            }
            Err(e) => Err(e),
        }
    }
    async fn get_one<T: Base + StoreTrait + RequestBody + DeserializeExt + 'static>(
        &mut self,
        id: &str,
    ) -> Result<T, Error> {
        let url = self.get_artifact_url(id);
        info!("get_one {}", url);
        match self.get_request::<T>(&url).await {
            Ok(mut r) => {
                trace!("{:?}", r);
                r.set_id(id);
                #[cfg(feature = "store")]
                let c = self.get_config();
                #[cfg(feature = "store")]
                let prepare_for_store = self.get_base_id() + "_" + id;
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
        info!("get_all {}", r);
        match self.get_vec(&r).await {
            Ok(mut v) => {
                v.satisfy(t);
                trace!("Response: {:?}", v);
                Ok(v)
            }
            Err(e) => Err(e),
        }
    }
    async fn get_one<T: Base + RequestBody + DeserializeExt + 'static>(
        &mut self,
        id: &str,
    ) -> Result<T, Error> {
        let url = self.get_artifact_url(id);
        info!("get_one {}", url);
        match self.get_request::<T>(&url).await {
            Ok(mut r) => {
                r.set_id(id);
                Ok(r)
            }
            Err(e) => Err(e),
        }
    }
}

#[async_trait]
pub trait Operation: ArtifactApi + HttpClient {
    async fn create<U: RequestBody, T: Send + DeserializeExt + 'static>(
        &mut self,
        body: &U,
    ) -> Result<T, Error> {
        let r = self.get_artifacts_url().to_owned();
        info!("create {}", r);
        self.post_request(&r, body).await
    }
    async fn delete<T: Deleted + DeserializeExt>(&mut self, id: &str) -> Result<T, Error> {
        let url = self.get_artifact_url(id);
        info!("delete {}", url);
        self.delete_request(&url).await
    }
    async fn put<B: RequestBody, T: Send + DeserializeExt + 'static>(
        &mut self,
        body: &B,
    ) -> Result<T, Error> {
        let url = self.get_artifact_url(&body.get_id());
        info!("put {}", url);
        self.put_request(&url, body).await
    }
}
