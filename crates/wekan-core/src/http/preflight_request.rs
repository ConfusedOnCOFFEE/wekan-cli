use crate::{config::AddressConfig, error::Error};
use async_trait::async_trait;
#[cfg(not(test))]
use log::{debug, trace};
#[cfg(not(test))]
use reqwest::{header::HeaderMap, Client as ReqClient, Response};

#[async_trait]
pub trait HealthCheck: AddressConfig {
    #[cfg(test)]
    async fn healthcheck(&mut self) -> Result<(), Error> {
        Ok(())
    }
    #[cfg(not(test))]
    async fn healthcheck(&mut self) -> Result<Response, Error> {
        let host_url = self.get_address();
        debug!("host_online");
        let headers = HeaderMap::new();
        match ReqClient::builder()
            .default_headers(headers)
            .build()?
            .get(host_url)
            .send()
            .await
        {
            Ok(ok) => Ok(ok),
            Err(e) => {
                trace!("{:?}", e);
                Err(Error::Http(e))
            }
        }
    }
}
