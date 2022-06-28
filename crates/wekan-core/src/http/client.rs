use super::{
    authentication::{Header, TokenManager},
    preflight_request::HealthCheck,
};
use crate::config::{AddressConfig, UserConfig};
use crate::error::Error;
use async_trait::async_trait;
use log::{info, trace};
#[cfg(not(test))]
use reqwest::Response as RResponse;
use serde::de::DeserializeOwned;
use wekan_common::{
    artifact::common::{Base, DeserializeExt},
    http::artifact::{RequestBody, Response},
};

#[cfg(test)]
use wekan_common::artifact::tests::MockResponse;

#[cfg(test)]
use wekan_common::http::artifact::Deleted;

#[cfg(test)]
use crate::http::operation::Artifacts;
#[derive(Debug, Clone)]
pub struct Client {
    pub config: UserConfig,
}

impl Client {
    pub fn new(config: UserConfig) -> Self {
        Client { config }
    }
}

impl AddressConfig for Client {
    fn get_address(&self) -> String {
        self.config.get_address()
    }

    fn get_api_address(&self) -> String {
        self.config.get_api_address()
    }
}

impl HealthCheck for Client {}

#[async_trait]
pub trait HttpClient: TokenManager + Header {
    async fn get_vec<'a, T: RequestBody + Clone + Base + DeserializeExt + 'static>(
        &mut self,
        url: &str,
    ) -> Result<Vec<T>, Error> {
        match self.get_request::<Response<T>>(url).await {
            Ok(res) => Ok(res.array.to_vec()),
            Err(e) => Err(e),
        }
    }
    async fn get_request<T: DeserializeExt>(&mut self, url: &str) -> Result<T, Error> {
        self.header().await?.get_ext::<T>(url).await
    }

    async fn post_request<T: RequestBody, U: DeserializeExt>(
        &mut self,
        url: &str,
        body: &T,
    ) -> Result<U, Error> {
        info!("post_request");
        trace!("{:?}", body);
        self.header().await?.post_ext::<T, U>(url, body).await
    }

    async fn delete_request<R: DeserializeExt>(&mut self, url: &str) -> Result<R, Error> {
        info!("delete_request");
        trace!("{:?}", url);
        self.header().await?.delete_ext::<R>(url).await
    }

    async fn put_request<B: RequestBody, U: DeserializeExt>(
        &mut self,
        url: &str,
        body: &B,
    ) -> Result<U, Error> {
        info!("put_request");
        self.header().await?.put_ext::<B, U>(url, body).await
    }
}

#[async_trait]
pub trait MethodMiddleware {
    async fn get_ext<B: DeserializeExt>(&self, url: &str) -> Result<B, Error>;
    async fn post_ext<B: RequestBody, U: DeserializeExt>(
        &self,
        url: &str,
        body: &B,
    ) -> Result<U, Error>;
    async fn delete_ext<B: DeserializeExt>(&self, url: &str) -> Result<B, Error>;
    async fn put_ext<B: RequestBody, U: DeserializeExt>(
        &self,
        url: &str,
        body: &B,
    ) -> Result<U, Error>;
}

#[async_trait]
#[cfg(test)]
impl MethodMiddleware for super::client::tests::MockClient {
    async fn get_ext<R: DeserializeExt>(&self, _url: &str) -> Result<R, Error> {
        Ok(R::mock())
    }
    async fn post_ext<B: RequestBody, U: DeserializeExt>(
        &self,
        _url: &str,
        _body: &B,
    ) -> Result<U, Error> {
        Ok(U::mock())
    }
    async fn delete_ext<R: DeserializeExt>(&self, _url: &str) -> Result<R, Error> {
        Ok(R::mock())
    }

    async fn put_ext<B: RequestBody, U: MockResponse>(
        &self,
        _url: &str,
        _body: &B,
    ) -> Result<U, Error> {
        Ok(U::mock())
    }
}

#[cfg(not(test))]
#[async_trait]
trait BuilderMiddleware {
    async fn send_ext(mut self) -> Result<RResponse, Error>;
    async fn form_json_ext<B: RequestBody, U: DeserializeOwned>(
        mut self,
        body: &B,
    ) -> Result<U, Error>;
    async fn deserialize_ext<U: DeserializeOwned>(mut self) -> Result<U, Error>;
    async fn form_ext<B: RequestBody>(mut self, body: &B) -> Result<RResponse, Error>;
}
#[cfg(not(test))]
#[async_trait]
impl MethodMiddleware for reqwest::Client {
    async fn get_ext<R: DeserializeExt>(&self, url: &str) -> Result<R, Error> {
        self.get(url).deserialize_ext::<R>().await
    }
    async fn post_ext<B: RequestBody, U: DeserializeExt>(
        &self,
        url: &str,
        body: &B,
    ) -> Result<U, Error> {
        self.post(url).form_json_ext::<B, U>(body).await
    }
    async fn delete_ext<B: DeserializeExt>(&self, url: &str) -> Result<B, Error> {
        self.delete(url).deserialize_ext::<B>().await
    }

    async fn put_ext<B: RequestBody, U: DeserializeExt>(
        &self,
        url: &str,
        body: &B,
    ) -> Result<U, Error> {
        self.put(url).form_json_ext::<B, U>(body).await
    }
}

#[cfg(not(test))]
#[async_trait]
impl BuilderMiddleware for reqwest::RequestBuilder {
    async fn deserialize_ext<R: DeserializeOwned>(mut self) -> Result<R, Error> {
        match self.send_ext().await?.json::<R>().await {
            Ok(ok) => Ok(ok),
            Err(e) => {
                trace!("{:?}", e);
                Err(Error::Http(e))
            }
        }
    }
    async fn form_json_ext<B: RequestBody, R: DeserializeOwned>(
        mut self,
        body: &B,
    ) -> Result<R, Error> {
        match self.form(&body).send_ext().await?.json::<R>().await {
            Ok(ok) => Ok(ok),
            Err(e) => {
                trace!("{:?}", e);
                Err(Error::Http(e))
            }
        }
    }
    async fn form_ext<B: RequestBody>(mut self, body: &B) -> Result<RResponse, Error> {
        self.form(&body).send_ext().await
    }
    async fn send_ext(mut self) -> Result<RResponse, Error> {
        match self.send().await {
            Ok(ok) => Ok(ok),
            Err(e) => Err(Error::Http(e)),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::{
        client::{BoardApi, Client},
        config::{ArtifactApi, MandatoryConfig, UserConfig},
    };
    use serde::{Deserialize, Serialize};
    use wekan_common::{
        artifact::common::{AType, IdReturner},
        http::artifact::RequestBody,
        validation::authentication::{Token, TokenHeader},
    };
    #[derive(Clone, Deserialize, Serialize, Debug)]
    pub struct MResponse {
        _id: String,
    }
    impl IdReturner for MResponse {
        fn get_id(&self) -> String {
            self._id.to_owned()
        }
    }
    impl RequestBody for MResponse {}
    impl Deleted for MResponse {}
    pub struct MockClient {}
    impl MockResponse for MResponse {
        fn mock() -> Self {
            MResponse {
                _id: String::from("fake-mock-response-id"),
            }
        }
    }
    pub trait ConvertVec {
        fn to_vec<T: std::fmt::Debug + DeserializeOwned + 'static>(&self) -> Vec<T> {
            Vec::new()
        }
    }
    impl ConvertVec for MResponse {}
    #[test]
    fn new_client() {
        let userconfig = UserConfig::new();
        let client = Client::new(userconfig);
        assert_eq!(
            client.get_artifacts_url(),
            "http://localhost:8080/api/boards/"
        );
    }

    #[tokio::test]
    async fn new_vec() {
        let userconfig = UserConfig::new();
        let mut client = Client::new(userconfig);
        client.set_token(Token {
            id: Box::new(String::from("B8D3e2qeXitTeqm9s")),
            token: Box::new(String::from("yNa1VR1Cz6nTzNirWPm2dRNYjdu-EM6LxKDIT0pIYsi")),
            token_expires: Box::new(String::from("2022-08-30T19:37:47.170Z")),
        });
        let v = client.get_all(AType::Board).await.unwrap();
        assert_eq!(v.len(), 0)
    }
}
