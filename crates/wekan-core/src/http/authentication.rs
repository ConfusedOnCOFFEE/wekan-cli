use async_trait::async_trait;
use chrono::prelude::*;
use chrono::DateTime;
use log::{debug, error, trace};
use reqwest::{
    header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    Client as ReqClient, Response as RResponse,
};
use wekan_common::validation::authentication::{Credentials, Token, TokenHeader};

use crate::{
    config::AddressConfig,
    error::kind::{CoreOk, Error},
};

#[cfg(feature = "store")]
use log::info;
#[cfg(feature = "store")]
use wekan_common::validation::authentication::StoreToken;

#[async_trait]
pub trait Login: AddressConfig {
    async fn request_token(&mut self, credentials: Credentials) -> Result<Token, Error> {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/x-www-form-urlencoded"),
        );
        let client = ReqClient::builder().default_headers(headers).build()?;
        let params = [("username", credentials.user), ("password", credentials.pw)];
        debug!("Login credentials={:?}", params);
        let url = self.get_address() + "/users/login";
        match client
            .post(url.to_owned())
            .form(&params)
            .send()
            .await?
            .json::<Token>()
            .await
        {
            Ok(res) => {
                debug!("login response:");
                trace!("{:?}", res);
                Ok(res)
            }
            Err(e) => Err(Error::Http(e)),
        }
    }
}

#[async_trait]
#[cfg(feature = "store")]
pub trait TokenManager: StoreToken + TokenHeader + Login {
    async fn request_valid_token(&mut self) -> Result<CoreOk, Error> {
        debug!("request_valid_token");
        let t = self.get_usertoken();
        trace!("{:?}", t);
        let expires = &*t.token_expires;
        trace!("{:?}", expires);
        if !expires.is_empty() {
            let utc: DateTime<Utc> = Utc::now();
            trace!("{:?}", expires);
            match expires.parse::<DateTime<Utc>>() {
                Ok(o) => {
                    trace!("Parsing match: {:?} vs {:?}", o, utc);
                    if o < utc {
                        self.renew_token().await
                    } else {
                        trace!("Token still valid");
                        Ok(CoreOk {
                            name: "Token still valid.".to_string(),
                        })
                    }
                }
                Err(e) => {
                    error!("Parsing didn't work. {:?}", e);
                    self.renew_token().await
                }
            }
        } else {
            error!("Token expires is empty.");
            self.renew_token().await
        }
    }
    async fn renew_token(&mut self) -> Result<CoreOk, Error> {
        info!("renew");
        match self.login(None).await {
            Ok(t) => {
                debug!("Login Token received.");
                trace!("{:?}", t);
                #[cfg(feature = "store")]
                self.store_token(t.clone()).await;
                self.set_token(t);
                Ok(CoreOk {
                    name: "Login succesful.".to_string(),
                })
            }
            Err(e) => Err(e),
        }
    }

    async fn login(&mut self, credentials: Option<Credentials>) -> Result<Token, Error> {
        info!("login with store");
        match credentials {
            Some(cr) => match self.request_token(cr).await {
                Ok(t) => Ok(self.store_token(t.clone()).await),
                Err(e) => Err(e),
            },
            None => match self.request_valid_token().await {
                Ok(_t) => Ok(self.get_usertoken()),
                Err(e) => Err(e),
            },
        }
    }

    async fn header(&mut self) -> Result<ReqClient, Error> {
        info!("header");
        match self.request_valid_token().await {
            Ok(_t) => {
                let mut headers = HeaderMap::new();
                headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
                let bearer_token = "Bearer ".to_owned() + &self.get_token();
                let mut token_header = HeaderValue::from_str(bearer_token.as_str()).unwrap();
                token_header.set_sensitive(true);
                headers.insert(AUTHORIZATION, token_header);
                match ReqClient::builder().default_headers(headers).build() {
                    Ok(o) => Ok(o),
                    Err(e) => Err(Error::Http(e)),
                }
            }
            Err(e) => Err(e),
        }
    }
}

#[async_trait]
#[cfg(not(feature = "store"))]
pub trait TokenManager: TokenHeader + Login {
    async fn request_valid_token(&mut self) -> Result<CoreOk, Error> {
        debug!("request_valid_token");
        let t = self.get_usertoken();
        let expires = &*t.token_expires;
        trace!("{:?}", expires);
        if !expires.is_empty() {
            let utc: DateTime<Utc> = Utc::now();
            trace!("{:?}", expires);
            match expires.parse::<DateTime<Utc>>() {
                Ok(o) => {
                    trace!("Parsing match: {:?} vs {:?}", o, utc);
                    if o < utc {
                        self.renew_token().await
                    } else {
                        trace!("Token still valid");
                        Ok(CoreOk {
                            name: "Token still valid.".to_string(),
                        })
                    }
                }
                Err(e) => {
                    error!("Parsing didn't work. {:?}", e);
                    self.renew_token().await
                }
            }
        } else {
            error!("Token expires is empty.");
            self.renew_token().await
        }
    }
    async fn renew_token(&mut self) -> Result<CoreOk, Error> {
        match self.login(None).await {
            Ok(t) => {
                debug!("Login Token received.");
                trace!("{:?}", t);
                #[cfg(feature = "store")]
                self.store_token(t).await;
                self.set_token(t);
                Ok(CoreOk {
                    name: "Login succesful.".to_string(),
                })
            }
            Err(e) => Err(e),
        }
    }

    async fn login(&mut self, credentials: Option<Credentials>) -> Result<Token, Error> {
        debug!("login without store");
        match credentials {
            Some(cr) => match self.request_token(cr).await {
                Ok(t) => Ok(self.set_token(t)),
                Err(e) => Err(e),
            },
            None => match self.request_valid_token().await {
                Ok(_t) => Ok(self.get_usertoken()),
                Err(e) => Err(e),
            },
        }
    }

    async fn header(&mut self) -> Result<ReqClient, Error> {
        match self.request_valid_token().await {
            Ok(_t) => {
                let mut headers = HeaderMap::new();
                headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
                let bearer_token = "Bearer ".to_owned() + &self.get_token();
                let mut token_header = HeaderValue::from_str(bearer_token.as_str()).unwrap();
                token_header.set_sensitive(true);
                headers.insert(AUTHORIZATION, token_header);
                match ReqClient::builder().default_headers(headers).build() {
                    Ok(o) => Ok(o),
                    Err(e) => Err(Error::Http(e)),
                }
            }
            Err(e) => Err(e),
        }
    }
}

#[async_trait]
pub trait Unauthorized {
    async fn builder() -> Result<ReqClient, Error> {
        let headers = HeaderMap::new();
        match ReqClient::builder().default_headers(headers).build() {
            Ok(o) => Ok(o),
            Err(e) => Err(Error::Http(e)),
        }
    }
    async fn get(url: &str) -> Result<RResponse, Error> {
        match Self::builder().await?.get(url).send().await {
            Ok(ok) => Ok(ok),
            Err(e) => Err(Error::Http(e)),
        }
    }
}
