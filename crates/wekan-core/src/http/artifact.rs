use async_trait::async_trait;
use log::{debug, info, trace};
use serde::{de::DeserializeOwned, Serialize};
use wekan_common::{
    artifact::common::{AType, Artifact, Base, SortedArtifact},
    http::artifact::Response
};

use crate::{
    error::kind::Error,
};

use super::client::HttpClient;

#[async_trait]
pub trait ArtifactClient: HttpClient {
    async fn get_vec<'a, T: std::fmt::Debug + Base + Clone + DeserializeOwned + 'static>(
        &mut self,
        url: &str,
    ) -> Result<Vec<T>, Error> {
        match self.get(url).await?.json::<Response<T>>().await {
            Ok(res) => Ok(res.array.to_vec()),
            Err(e) => Err(Error::Http(e)),
        }
    }
    async fn get_details<'a, T: std::fmt::Debug + DeserializeOwned + 'static>(
        &mut self,
        url: &str,
    ) -> Result<T, Error> {
        match self.get(url).await?.json::<T>().await {
            Ok(res) => Ok(res),
            Err(e) => Err(Error::Http(e)),
        }
    }

    async fn get_string(&mut self, url: &str) -> Result<String, Error> {
        println!("{}", url);
        match self.get(url).await?.text().await {
            Ok(r) => {
                debug!("post_artifact response {:?}", r);
                Ok(r)
            }
            Err(e) => Err(Error::Http(e)),
        }
    }

    async fn post_artifact<
        'a,
        T: std::marker::Sync + std::fmt::Debug + Serialize + Send + Clone,
        U: std::fmt::Debug + DeserializeOwned + 'static,
    >(
        &mut self,
        url: &str,
        body: &T,
    ) -> Result<U, Error> {
        info!("post_artifact");
        debug!("{:?}", body);
        match self.post(url, body).await?.json::<U>().await {
            Ok(res) => {
                debug!("post_artifact response {:?}", res);
                Ok(res)
            }
            Err(e) => Err(Error::Http(e)),
        }
    }

    async fn delete_artifact<'a, U: std::fmt::Debug + DeserializeOwned + 'static>(
        &mut self,
        url: &str,
    ) -> Result<U, Error> {
        match self.delete(url).await?.json::<U>().await {
            Ok(res) => {
                debug!("post_artifact response {:?}", res);
                Ok(res)
            }
            Err(e) => Err(Error::Http(e)),
        }
    }

    async fn put_artifact<
        'a,
        T: std::marker::Sync + std::fmt::Debug + Serialize + Send + Clone,
        U: std::fmt::Debug + DeserializeOwned + 'static,
    >(
        &mut self,
        url: &str,
        body: &T,
    ) -> Result<U, Error> {
        match self.put(url, body).await?.json::<U>().await {
            Ok(res) => {
                debug!("put_artifact response {:?}", res);
                Ok(res)
            }
            Err(e) => {
                trace!("put_artifact: {:?}", e);
                Err(Error::Http(e))
            },
        }
    }
}

#[async_trait]
pub trait Unwrapper {
    async fn get_result(res: Result<Vec<Artifact>, Error>) -> Vec<Artifact> {
        match res {
            Ok(res) => res,
            Err(_e) => Vec::<Artifact>::new(),
        }
    }
}

pub trait SatisfyType {
    fn satisfy(&mut self, atype: AType);
}
impl<A: SortedArtifact + std::fmt::Debug> SatisfyType for Vec<A> {
    fn satisfy(&mut self, atype: AType) {
        let iter = self.iter_mut();
        for el in iter {
            el.set_type(atype.clone());
        }
        debug!("Response: {:?}", self);
    }
}
