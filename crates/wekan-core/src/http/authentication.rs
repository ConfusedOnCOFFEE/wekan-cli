use crate::{
    config::AddressConfig,
    error::kind::{CoreOk, Error},
};

use async_trait::async_trait;
use chrono::prelude::*;
use chrono::DateTime;
use log::{debug, error, trace};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION};
#[cfg(not(test))]
use reqwest::{header::CONTENT_TYPE, Client as ReqClient, Response as RResponse};
use wekan_common::validation::authentication::{Credentials, Token, TokenHeader};

#[cfg(test)]
use crate::http::client::tests::{MResponse, MockClient};
use crate::http::client::MethodMiddleware;

use log::info;
use wekan_common::validation::authentication::StoreToken;

#[async_trait]
pub trait Login: AddressConfig {
    async fn request_token(&mut self, credentials: Credentials) -> Result<Token, Error> {
        let params = [("username", credentials.user), ("password", credentials.pw)];
        debug!("Login credentials={:?}", params);
        let url = self.get_address() + "/users/login";
        self.post_login_form(&url, params).await
    }
    #[cfg(not(test))]
    async fn post_login_form(
        &mut self,
        url: &str,
        params: [(&str, String); 2],
    ) -> Result<Token, Error> {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/x-www-form-urlencoded"),
        );
        let client = ReqClient::builder().default_headers(headers).build()?;
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
    #[cfg(test)]
    async fn post_login_form(
        &mut self,
        _url: &str,
        _params: [(&str, String); 2],
    ) -> Result<Token, Error> {
        Ok(Token {
            id: Box::new(String::from("B8D3e2qeXitTeqm9s")),
            token: Box::new(String::from("yNa1VR1Cz6nTzNirWPm2dRNYjdu-EM6LxKDIT0pIYsi")),
            token_expires: Box::new(String::from("2022-08-30T19:37:47.170Z")),
        })
    }
}

#[async_trait]
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
        info!("renew_token");
        match self.login(None).await {
            Ok(t) => {
                debug!("Login Token received.");
                trace!("{:?}", t);
                #[cfg(feature = "store")]
                self.store_token(t.clone()).await;
                self.set_token(t);
                Ok(CoreOk {
                    name: "Login succes.".to_string(),
                })
            }
            Err(e) => Err(e),
        }
    }

    async fn login(&mut self, credentials: Option<Credentials>) -> Result<Token, Error> {
        match credentials {
            Some(cr) => match self.request_token(cr).await {
                Ok(t) => {
                    #[cfg(feature = "store")]
                    self.store_token(t.clone()).await;
                    Ok(t)
                }
                Err(e) => Err(e),
            },
            None => match self.request_valid_token().await {
                Ok(_t) => Ok(self.get_usertoken()),
                Err(e) => Err(e),
            },
        }
    }
}

#[async_trait]
pub trait Header {
    type Client: MethodMiddleware + std::marker::Send + std::marker::Sync;
    async fn header(&mut self) -> Result<Self::Client, Error>;
}

#[cfg(not(test))]
#[async_trait]
impl Header for crate::client::Client {
    type Client = ReqClient;
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

#[cfg(test)]
#[async_trait]
impl Header for crate::client::Client {
    type Client = MockClient;
    async fn header(&mut self) -> Result<MockClient, Error> {
        match self.request_valid_token().await {
            Ok(_t) => {
                let mut headers = HeaderMap::new();
                headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
                let bearer_token = "Bearer ".to_owned() + &self.get_token();
                let mut token_header = HeaderValue::from_str(bearer_token.as_str()).unwrap();
                token_header.set_sensitive(true);
                headers.insert(AUTHORIZATION, token_header);
                Ok(MockClient {})
            }
            Err(e) => Err(e),
        }
    }
}

#[async_trait]
#[cfg(test)]
pub trait Unauthorized {
    fn builder() -> Result<MockClient, Error> {
        Ok(MockClient {})
    }
    async fn get(_url: &str) -> Result<MResponse, Error> {
        Ok(MResponse {})
    }
}

#[async_trait]
#[cfg(not(test))]
pub trait Unauthorized {
    fn builder() -> Result<ReqClient, Error> {
        let headers = HeaderMap::new();
        match ReqClient::builder().default_headers(headers).build() {
            Ok(o) => Ok(o),
            Err(e) => Err(Error::Http(e)),
        }
    }
    async fn get(url: &str) -> Result<RResponse, Error> {
        match Self::builder()?.get(url).send().await {
            Ok(ok) => Ok(ok),
            Err(e) => Err(Error::Http(e)),
        }
    }
}
