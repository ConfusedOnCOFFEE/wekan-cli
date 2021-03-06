use crate::{
    config::{AddressConfig, ArtifactApi, ConfigRequester, OptionalConfig, UserConfig},
    http::{
        authentication::{Login, TokenManager},
        client::HttpClient,
        operation::{Artifacts, Operation},
        preflight_request::HealthCheck,
    },
};
use async_trait::async_trait;
use wekan_common::validation::authentication::{StoreToken, Token, TokenHeader};
use wekan_core_derive::{ArtifactClient, TokenConfig, TokenManagerClient};

#[derive(TokenManagerClient, TokenConfig, Clone, Debug)]
pub struct LoginClient {
    pub config: UserConfig,
}

impl LoginClient {
    pub fn new(config: UserConfig) -> Self {
        LoginClient { config }
    }
}

#[derive(ArtifactClient, TokenConfig, Clone, Debug)]
pub struct Client {
    pub config: UserConfig,
    base: String,
    pub id: String,
}

impl ConfigRequester<UserConfig> for Client {
    fn get_config(&self) -> UserConfig {
        self.config.get_config()
    }
    fn get_base_id(&self) -> String {
        self.id.to_owned()
    }
}

pub trait BoardApi {
    fn new(config: UserConfig) -> Self;
    fn set_base(&mut self, base: &str) -> String;
}

impl BoardApi for Client {
    fn new(config: UserConfig) -> Self {
        Self {
            config,
            base: String::from("boards/"),
            id: String::new(),
        }
    }
    fn set_base(&mut self, base: &str) -> String {
        self.base = base.to_string();
        self.base.to_owned()
    }
}

pub trait ListApi {
    fn new(config: UserConfig, board_id: &str) -> Self;
    fn set_base(&mut self, board_id: &str) -> String;
}

impl ListApi for Client {
    fn new(config: UserConfig, board_id: &str) -> Self {
        Self {
            config,
            base: "boards/".to_owned() + board_id + "/lists/",
            id: board_id.to_string(),
        }
    }
    fn set_base(&mut self, board_id: &str) -> String {
        self.base = "boards/".to_owned() + board_id + "/lists/";
        self.base.to_owned()
    }
}

pub trait SwimlaneApi {
    fn new(config: UserConfig, board_id: &str) -> Self;
    fn set_base(&mut self, board_id: &str) -> String;
}

impl SwimlaneApi for Client {
    fn new(config: UserConfig, board_id: &str) -> Self {
        Self {
            config,
            base: "boards/".to_owned() + board_id + "/swimlanes/",
            id: board_id.to_string(),
        }
    }
    fn set_base(&mut self, board_id: &str) -> String {
        self.base = "boards/".to_owned() + board_id + "/swimlanes/";
        self.base.to_owned()
    }
}

pub trait CardApi {
    fn new(config: UserConfig, board_id: &str, list_id: &str) -> Self;
    fn set_base(&mut self, board_id: &str, list_id: &str) -> String;
}
impl CardApi for Client {
    fn new(config: UserConfig, board_id: &str, list_id: &str) -> Self {
        Self {
            config,
            base: "boards/".to_owned() + board_id + "/lists/" + list_id + "/cards/",
            id: board_id.to_owned() + "_" + list_id,
        }
    }
    fn set_base(&mut self, board_id: &str, list_id: &str) -> String {
        self.base = "boards/".to_owned() + board_id + "/lists/" + list_id + "/cards/";
        self.base.to_owned()
    }
}

pub trait ChecklistApi {
    fn new(config: UserConfig, board_id: &str, card_id: &str) -> Self;
    fn set_base(&mut self, card_id: &str, card_id: &str) -> String;
}
impl ChecklistApi for Client {
    fn new(config: UserConfig, board_id: &str, card_id: &str) -> Self {
        Self {
            config,
            base: "boards/".to_owned() + board_id + "/cards/" + card_id + "/checklists/",
            id: board_id.to_owned() + "_" + card_id,
        }
    }
    fn set_base(&mut self, board_id: &str, card_id: &str) -> String {
        self.base = "boards/".to_owned() + board_id + "/cards/" + card_id + "/checklists/";
        self.base.to_owned()
    }
}

pub trait ChecklistItemsApi {
    fn new(config: UserConfig, board_id: &str, card_id: &str, checklist_id: &str) -> Self;
    fn set_base(&mut self, card_id: &str, card_id: &str, checklist_id: &str) -> String;
}
impl ChecklistItemsApi for Client {
    fn new(config: UserConfig, board_id: &str, card_id: &str, checklist_id: &str) -> Self {
        Self {
            config,
            base: "boards/".to_owned()
                + board_id
                + "/cards/"
                + card_id
                + "/checklists/"
                + checklist_id
                + "/items/",
            id: board_id.to_owned() + "_" + card_id,
        }
    }
    fn set_base(&mut self, board_id: &str, card_id: &str, checklist_id: &str) -> String {
        self.base = "boards/".to_owned()
            + board_id
            + "/cards/"
            + card_id
            + "/checklists/"
            + checklist_id
            + "/items/";
        self.base.to_owned()
    }
}
