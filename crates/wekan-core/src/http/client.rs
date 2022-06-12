use super::{authentication::TokenManager, preflight_request::Client as PFRClient};
use crate::config::{AddressConfig, UserConfig};
use crate::error::kind::Error;
use async_trait::async_trait;
use log::{debug, trace};
use reqwest::Response as RResponse;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Clone)]
#[serde(transparent)]
pub struct Response<T> {
    pub array: Vec<T>,
}

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

impl PFRClient for Client {}

#[async_trait]
pub trait HttpClient: TokenManager {
    async fn get(&mut self, url: &str) -> Result<RResponse, Error> {
        match self.header().await?.get(url).send().await {
            Ok(ok) => Ok(ok),
            Err(e) => Err(Error::Http(e)),
        }
    }

    async fn post<'a, T: std::marker::Sync + std::fmt::Debug + Serialize + Send>(
        &mut self,
        url: &str,
        body: &T,
    ) -> Result<RResponse, Error> {
        debug!("post");
        trace!("{:?}", body);
        match self.header().await?.post(url).json(body).send().await {
            Ok(ok) => Ok(ok),
            Err(e) => Err(Error::Http(e)),
        }
    }

    async fn delete(&mut self, url: &str) -> Result<RResponse, Error> {
        debug!("delete");
        trace!("{:?}", url);
        match self.header().await?.delete(url).send().await {
            Ok(ok) => Ok(ok),
            Err(e) => Err(Error::Http(e)),
        }
    }

    async fn put<'a, T: std::marker::Sync + std::fmt::Debug + Serialize + Send>(
        &mut self,
        url: &str,
        body: &T,
    ) -> Result<RResponse, Error> {
        trace!("put-URL: {:?}", url);
        trace!("put-BODY: {:?}", body);
        match self.header().await?.put(url).form(body).send().await {
            Ok(ok) => Ok(ok),
            Err(e) => {
                trace!("client.put: {:?}", e);
                Err(Error::Http(e))
            }
        }
    }

    async fn update<'a, T: std::marker::Sync + std::fmt::Debug + Serialize + Send>(
        &mut self,
        url: &str,
        body: &T,
    ) -> Result<RResponse, Error> {
        trace!("update: {:?}", url);
        match self.header().await?.put(url).form(body).send().await {
            Ok(ok) => Ok(ok),
            Err(e) => {
                trace!("{:?}", e);
                Err(Error::Http(e))
            }
        }
    }
}
