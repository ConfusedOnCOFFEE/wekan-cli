use async_trait::async_trait;
use log::{debug, trace};

use crate::{error::kind::Error, config::AddressConfig};
use reqwest::{header::HeaderMap, Client as ReqClient, Response};

#[async_trait]
pub trait Client: AddressConfig {
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
