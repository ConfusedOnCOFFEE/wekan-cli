use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct Token {
    pub id: Box<String>,
    pub token: Box<String>,
    pub token_expires: Box<String>,
}

pub struct Credentials {
    pub user: String,
    pub pw: String,
}

pub trait TokenHeader {
    fn get_usertoken(&self) -> Token;
    fn get_token(&self) -> String;
    fn set_token(&mut self, t: Token) -> Token;
    fn get_user_id(&self) -> String;
}

#[async_trait]
pub trait StoreToken {
    async fn store_token(&mut self, t: Token) -> Token;
}
