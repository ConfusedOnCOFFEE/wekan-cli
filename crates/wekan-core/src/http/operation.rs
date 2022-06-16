use super::util::SatisfyType;
#[cfg(feature = "store")]
use crate::{
    client::Client,
    config::{ConfigRequester, UserConfig},
    persistence::store::Store,
};
use crate::{config::ArtifactApi, error::kind::Error, http::client::HttpClient};
use async_trait::async_trait;
use log::debug;
#[cfg(feature = "store")]
use log::trace;
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
    async fn get_one<T: Base + StoreTrait + RequestBody + DeserializeExt + 'static>(
        &mut self,
        id: &str,
    ) -> Result<T, Error> {
        let url = self.get_artifact_url(id);
        debug!("get_one {:?}", url);
        match self.get_request::<T>(&url).await {
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
    async fn get_one<T: Base + RequestBody + DeserializeExt + 'static>(
        &mut self,
        id: &str,
    ) -> Result<T, Error> {
        let url = self.get_artifact_url(id);
        debug!("get_one {:?}", url);
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
        debug!("create {:?}", r);
        self.post_request(&r, body).await
    }
    async fn delete<T: Deleted + DeserializeExt>(&mut self, id: &str) -> Result<T, Error> {
        let url = self.get_artifact_url(id);
        debug!("delete {:?}", url);
        self.delete_request(&url).await
    }
    async fn put<U: RequestBody, T: Send + DeserializeExt + 'static>(
        &mut self,
        id: &str,
        body: &U,
    ) -> Result<T, Error> {
        let url = self.get_artifact_url(id);
        debug!("put {:?}", url);
        self.put_request(&url, body).await
    }
}

// #[cfg(test)]
// pub mod tests {
//     use super::*;
//     use wekan_common::artifact::tests::{MockNewResponse as NewResponse};
//     use crate::http::client::tests::MockResponse;

//     #[async_trait]
//     pub trait Operation: ArtifactApi {
//         async fn create<U: RequestBody, T: NewResponse + Send + Debug + DeserializeOwned + 'static>(
//             &mut self,
//             body: &U,
//         ) -> Result<T, Error> {
//             let r = self.get_artifacts_url().to_owned();
//             debug!("create {:?}", r);
//             self.post_artifact(&r, body).await
//         }
//         async fn delete<T: Deleted + RequestBody + NewResponse>(&mut self, id: &str) -> Result<T, Error> {
//             let url = self.get_artifact_url(id);
//             debug!("delete {:?}", url);
//             self.delete_artifact(&url).await
//         }
//         async fn put<U: RequestBody + NewResponse + Debug + DeserializeOwned + 'static>(
//             &mut self,
//             id: &str,
//             body: &U,
//         ) -> Result<U, Error> {
//             let url = self.get_artifact_url(id);
//             debug!("put {:?}", url);
//             self.put_artifact::<U>(&url, body).await
//         }
//     }

//     #[async_trait]
//     pub trait Artifacts: Operation {
//         async fn get_all(&mut self, t: AType) -> Result<Vec<Artifact>, Error> {
//             let r = self.get_artifacts_url().to_owned();
//             debug!("get_all {:?}", r);
//             match self.get_vec::<Artifact, MockResponse>(&r).await {
//                 Ok(mut v) => {
//                     v.satisfy(t);
//                     debug!("Response: {:?}", v);
//                     Ok(v)
//                 }
//                 Err(e) => Err(e),
//             }
//         }
//         async fn get_one<T: Base + RequestBody + NewResponse + DeserializeOwned + 'static>(
//             &mut self,
//             id: &str,
//         ) -> Result<T, Error> {
//             let url = self.get_artifact_url(id);
//             debug!("get_one {:?}", url);
//             match self.get_details::<T>(&url).await {
//                 Ok(mut r) => {
//                     r.set_id(id);
//                     Ok(r)
//                 }
//                 Err(e) => Err(e),
//             }
//         }
//     }
// }
