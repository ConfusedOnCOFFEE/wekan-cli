#[cfg(feature = "store")]
use crate::persistence::config::PersistenceConfig as PConfig;
use async_trait::async_trait;
use log::debug;
use serde::{Deserialize, Serialize};
use wekan_common::validation::authentication::Token;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct NetworkAddress {
    host: String,
    port: i16,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct UserConfig {
    pub address: NetworkAddress,
    pub context: Option<String>,
    pub usertoken: Option<Token>,
}

#[async_trait]
pub trait MandatoryConfig {
    fn new() -> Self;
    fn set_token(&mut self, t: Token);
}

#[async_trait]
pub trait Setup {
    fn set_host(&mut self, host: String);
    fn set_port(&mut self, port: i16);
}

pub trait AddressConfig {
    fn get_address(&self) -> String;
    fn get_api_address(&self) -> String;
}
#[async_trait]
pub trait OptionalConfig {
    async fn store_token(&mut self, t: Token) -> Token;
}

#[async_trait]
impl OptionalConfig for UserConfig {
    async fn store_token(&mut self, t: Token) -> Token {
        debug!("set_token");
        self.usertoken = Some(t);
        #[cfg(feature = "store")]
        self.write_config({
            UserConfig {
                address: self.address.clone(),
                usertoken: self.usertoken.clone(),
                context: self.context.clone(),
            }
        })
        .await;
        self.usertoken.as_ref().unwrap().to_owned()
    }
}

impl Setup for UserConfig {
    fn set_host(&mut self, host: String) {
        self.address.set_host(host);
    }

    fn set_port(&mut self, port: i16) {
        debug!("Port: {:?}", port);
        self.address.set_port(port);
    }
}

impl Setup for NetworkAddress {
    fn set_host(&mut self, host: String) {
        self.host = host;
    }

    fn set_port(&mut self, port: i16) {
        self.port = port;
    }
}

pub trait ArtifactApi {
    fn get_artifacts_url(&self) -> String;
    fn get_artifact_url(&self, id: &str) -> String;
}

impl AddressConfig for NetworkAddress {
    fn get_address(&self) -> String {
        self.host.to_owned() + ":" + &self.port.to_string()
    }

    fn get_api_address(&self) -> String {
        self.get_address() + "/api/"
    }
}

impl AddressConfig for UserConfig {
    fn get_address(&self) -> String {
        self.address.get_address()
    }

    fn get_api_address(&self) -> String {
        self.address.get_api_address()
    }
}

pub trait ConfigRequester<T> {
    fn get_config(&self) -> T;
    fn get_base_id(&self) -> String;
}

impl ConfigRequester<UserConfig> for UserConfig {
    fn get_config(&self) -> Self {
        self.to_owned()
    }
    fn get_base_id(&self) -> String {
        *self.usertoken.as_ref().unwrap().id.to_owned()
    }
}

#[async_trait]
impl MandatoryConfig for UserConfig {
    fn new() -> Self {
        UserConfig {
            address: NetworkAddress {
                host: String::from("http://localhost"),
                port: 8080,
            },
            context: None,
            usertoken: None,
        }
    }

    fn set_token(&mut self, t: Token) {
        self.usertoken = Some(t)
    }
}
