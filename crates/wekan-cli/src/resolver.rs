use log::{error, info, trace};
use regex::Regex;
use wekan_common::{
    artifact::common::{AType, Artifact, Base, IdReturner, SortedArtifact, WekanDisplay},
    validation::authentication::TokenHeader,
    validation::constraint::Constraint,
};
use wekan_core::{
    client::{BoardApi, CardApi, ChecklistApi, Client, ListApi, SwimlaneApi},
    config::UserConfig,
};

use crate::error::{CliError, Error, Transform};
#[cfg(feature = "store")]
use crate::store::Store;
#[cfg(test)]
use crate::tests::mocks::Artifacts;
#[cfg(feature = "store")]
use chrono::{prelude::*, DateTime};
#[cfg(not(test))]
use wekan_core::http::operation::Artifacts;

pub struct Query<'a> {
    pub config: UserConfig,
    pub filter: &'a str,
    #[cfg(feature = "store")]
    pub deny_store_usage: bool,
}

impl<'a> Query<'a> {
    pub async fn find_card_id(
        &mut self,
        board_id: &str,
        list_id: &str,
        name: &str,
    ) -> Result<String, Error> {
        info!("find_card_id");
        let cards = match self
            .inquire(AType::Card, Some(board_id), Some(list_id), false)
            .await
        {
            Ok(o) => Ok(o),
            Err(e) => {
                trace!("{:?}", e);
                self.fulfill_inquiry(AType::Card, Some(board_id), Some(list_id))
                    .await
            }
        };
        self.confirm_valid_name(cards, name).await
    }

    pub async fn find_swimlane_id(&mut self, board_id: &str) -> Result<String, Error> {
        info!("find_swimlane_id");
        let swimlane = match self
            .inquire(AType::Swimlane, Some(board_id), None, false)
            .await
        {
            Ok(o) => Ok(o),
            Err(e) => {
                trace!("{:?}", e);
                self.fulfill_inquiry(AType::Swimlane, Some(board_id), None)
                    .await
            }
        };
        self.confirm_valid_name(swimlane, &String::from("Default"))
            .await
    }

    pub async fn find_list_id(&mut self, board_id: &str, name: &str) -> Result<String, Error> {
        info!("find_list_id");
        let boards = match self.inquire(AType::List, Some(board_id), None, false).await {
            Ok(o) => Ok(o),
            Err(e) => {
                trace!("{:?}", e);
                self.fulfill_inquiry(AType::List, Some(board_id), None)
                    .await
            }
        };
        self.confirm_valid_name(boards, name).await
    }

    pub async fn find_board_id(&mut self, name: &str) -> Result<String, Error> {
        info!("find_board_id");
        let boards = match self.inquire(AType::Board, None, None, false).await {
            Ok(o) => Ok(o),
            Err(e) => {
                trace!("{:?}", e);
                self.fulfill_inquiry(AType::Board, None, None).await
            }
        };
        self.confirm_valid_name(boards, name).await
    }

    pub async fn find_checklist_id(
        &mut self,
        board_id: &str,
        card_id: &str,
        name: &str,
    ) -> Result<String, Error> {
        info!("find_board_id");
        let checklists = match self
            .inquire(AType::Checklist, Some(board_id), Some(card_id), false)
            .await
        {
            Ok(o) => Ok(o),
            Err(e) => {
                trace!("{:?}", e);
                self.fulfill_inquiry(AType::Checklist, Some(board_id), Some(card_id))
                    .await
            }
        };
        self.confirm_valid_name(checklists, name).await
    }

    #[cfg(not(feature = "store"))]
    pub async fn inquire(
        &self,
        artifact_variant: AType,
        board_id: Option<&str>,
        list_id: Option<&str>,
        _fresh_request: bool,
    ) -> Result<Vec<Artifact>, Error> {
        info!("inquire");
        self.fulfill_inquiry(artifact_variant, board_id, list_id)
            .await
    }

    #[cfg(feature = "store")]
    pub async fn inquire(
        &self,
        artifact_variant: AType,
        board_id: Option<&str>,
        list_id: Option<&str>,
        fresh_request: bool,
    ) -> Result<Vec<Artifact>, Error> {
        info!("inquire");
        if self.deny_store_usage || fresh_request {
            self.fulfill_inquiry(artifact_variant, board_id, list_id)
                .await
        } else {
            match artifact_variant {
                AType::Board => {
                    self.compare_age(artifact_variant, board_id, list_id, &15)
                        .await
                }
                AType::List => {
                    self.compare_age(artifact_variant, board_id, list_id, &10)
                        .await
                }
                AType::Card => {
                    self.compare_age(artifact_variant, board_id, list_id, &3)
                        .await
                }
                AType::Swimlane => {
                    self.compare_age(artifact_variant, board_id, list_id, &15)
                        .await
                }
                AType::Checklist => {
                    self.compare_age(artifact_variant, board_id, list_id, &15)
                        .await
                }
                _ => {
                    error!("Not a AType or empty list");
                    Ok(Vec::new())
                }
            }
        }
    }
    #[cfg(feature = "store")]
    async fn compare_age(
        &self,
        artifact_variant: AType,
        board_id: Option<&str>,
        list_id: Option<&str>,
        diff: &u32,
    ) -> Result<Vec<Artifact>, Error> {
        let join_ids = match board_id {
            Some(b) => match list_id {
                Some(l) => b.to_owned() + "_" + l,
                None => b.to_string(),
            },
            None => String::new(),
        };
        trace!("Joined ids: {}", join_ids);
        match self
            .lookup_artifacts(artifact_variant.clone(), &join_ids)
            .await
        {
            Ok(o) => match o.age.parse::<DateTime<Utc>>() {
                Ok(t) => {
                    let mut last_available_minute_of_store: u32 = t.minute() + *diff;
                    if t.minute() > 60 {
                        trace!("Over 60 => {}", t.minute());
                        last_available_minute_of_store = t.minute() - 60;
                    }
                    trace!(
                        "Compare equation: {} < {}",
                        last_available_minute_of_store,
                        Utc::now().minute()
                    );
                    if last_available_minute_of_store < Utc::now().minute() {
                        info!("Request");
                        self.fulfill_inquiry(artifact_variant, board_id, list_id)
                            .await
                    } else {
                        info!("Store");
                        Ok(o.payload)
                    }
                }
                Err(_e) => {
                    self.fulfill_inquiry(artifact_variant, board_id, list_id)
                        .await
                }
            },
            Err(_e) => {
                self.fulfill_inquiry(artifact_variant, board_id, list_id)
                    .await
            }
        }
    }

    async fn fulfill_inquiry(
        &self,
        artifact_variant: AType,
        board_id: Option<&str>,
        second_artifact_id: Option<&str>,
    ) -> Result<Vec<Artifact>, Error> {
        info!("fulfill_inquire");
        trace!("AType: {:?}", artifact_variant);
        match artifact_variant {
            AType::Board => self.request_boards().await,
            AType::List | AType::Swimlane | AType::Card | AType::Checklist => match board_id {
                Some(b) => match artifact_variant {
                    AType::List => self.request_lists(b).await,
                    AType::Swimlane => self.request_swimlanes(b).await,
                    AType::Card => match second_artifact_id {
                        Some(l) => self.request_cards(b, l).await,
                        None => panic!("List id needs to be supplied"),
                    },
                    AType::Checklist => match second_artifact_id {
                        Some(c) => self.request_checklists(b, c).await,
                        None => panic!("Card it needs to be supplied"),
                    },
                    _ => Ok(Vec::new()),
                },
                None => panic!("Board id needs to be supplied"),
            },
            _ => {
                error!("Not a AType or empty list");
                Ok(Vec::new())
            }
        }
    }
    async fn request_boards(&self) -> Result<Vec<Artifact>, Error> {
        info!("requests_board");
        let mut client = <Client as BoardApi>::new(self.config.to_owned());
        let user_id = client.get_user_id();
        BoardApi::set_base(&mut client, &("users/".to_owned() + &user_id + "/boards"));
        match client.get_all(AType::Board).await {
            Ok(o) => Ok(o),
            Err(e) => Err(Error::from(e)),
        }
    }

    async fn request_swimlanes(&self, board_id: &str) -> Result<Vec<Artifact>, Error> {
        info!("request_swimlanes");
        let mut client = <Client as SwimlaneApi>::new(self.config.to_owned(), board_id);
        match client.get_all(AType::Swimlane).await {
            Ok(o) => Ok(o),
            Err(e) => Err(Error::from(e)),
        }
    }

    async fn request_cards(&self, board_id: &str, list_id: &str) -> Result<Vec<Artifact>, Error> {
        info!("request_cards");
        let mut client = <Client as CardApi>::new(self.config.to_owned(), board_id, list_id);
        match client.get_all(AType::Card).await {
            Ok(o) => Ok(o),
            Err(e) => Err(Error::from(e)),
        }
    }
    async fn request_lists(&self, board_id: &str) -> Result<Vec<Artifact>, Error> {
        info!("request_lists");
        let mut client = <Client as ListApi>::new(self.config.to_owned(), board_id);
        match client.get_all(AType::List).await {
            Ok(o) => Ok(o),
            Err(e) => Err(Error::from(e)),
        }
    }
    async fn request_checklists(
        &self,
        board_id: &str,
        card_id: &str,
    ) -> Result<Vec<Artifact>, Error> {
        info!("request_cheklists");
        let mut client = <Client as ChecklistApi>::new(self.config.to_owned(), board_id, card_id);
        match client.get_all(AType::Checklist).await {
            Ok(o) => Ok(o),
            Err(e) => Err(Error::from(e)),
        }
    }
    async fn confirm_valid_name(
        &mut self,
        vecs: Result<Vec<impl WekanDisplay>, Error>,
        name: &str,
    ) -> Result<String, Error> {
        match vecs {
            Ok(r#as) => self.extract_id(r#as, name).await,
            Err(_e) => Err(CliError::new_msg("Artifact not found").as_enum()),
        }
    }

    async fn extract_id(
        &mut self,
        vecs: Vec<impl WekanDisplay>,
        name: &str,
    ) -> Result<String, Error> {
        let mut iter = vecs.iter();
        loop {
            match iter.next() {
                Some(artifact) => {
                    let a = Artifact {
                        _id: artifact.get_id(),
                        title: artifact.get_title(),
                        r#type: artifact.get_type(),
                    };
                    trace!("{:?} - {}", a, name);
                    if self.check_artifact(&a, name) {
                        info!("Artifact found");
                        break Ok(a.get_id());
                    }
                }
                None => break Err(CliError::new_msg("Artifact not found").as_enum()),
            };
        }
    }

    fn check_artifact(&self, artifact: &Artifact, name: &str) -> bool {
        if !self.filter.is_empty() && self.match_title_by_type(artifact) {
            true
        } else {
            Query::contains_title(artifact, name)
        }
    }

    fn match_title_by_type(&self, artifact: &Artifact) -> bool {
        match artifact.get_type() {
            AType::Board => {
                let b = Regex::new(r"b:(?P<board>[a-z0-9A-Z]{1,4}),l:.*").unwrap();
                let before = "b:a,l:Ej,c:I";
                let after = b.captures(before).unwrap();
                assert_eq!(after.name("board").unwrap().as_str(), "a");
                let before = "b:a8as,l:Eklk,c:I09";
                let after = b.captures(before).unwrap();
                assert_eq!(after.name("board").unwrap().as_str(), "a8as");
                self.match_captured_user_input(b, artifact)
            }
            AType::List => {
                let b =
                    Regex::new(r"b:[a-z0-9A-Z]{1,4},l:(?P<list>[a-z0-9A-Z]{1,4}),?c?:?.*").unwrap();
                let before = "b:a,l:Ej,c:I";
                let after = b.captures(before).unwrap();
                assert_eq!(after.name("list").unwrap().as_str(), "Ej");
                let before = "b:a8as,l:Eklk,c:I09";
                let after = b.captures(before).unwrap();
                assert_eq!(after.name("list").unwrap().as_str(), "Eklk");
                self.match_captured_user_input(b, artifact)
            }
            AType::Card => {
                let b = Regex::new(
                    r"b:[a-z0-9A-Z]{1,4},l:[a-z0-9A-Z]{1,4},c:(?P<card>[a-z0-9A-Z]{1,4})",
                )
                .unwrap();
                let before = "b:a,l:Ej,c:I";
                let after = b.captures(before).unwrap();
                assert_eq!(after.name("card").unwrap().as_str(), "I");
                let before = "b:a8as,l:Eklk,c:I09";
                let after = b.captures(before).unwrap();
                assert_eq!(after.name("card").unwrap().as_str(), "I09");
                self.match_captured_user_input(b, artifact)
            }
            _ => false,
        }
    }

    fn match_captured_user_input(&self, regex: Regex, artifact: &Artifact) -> bool {
        match regex.captures(self.filter) {
            Some(s) => {
                trace!("{:?}", s);
                match s.name("card") {
                    Some(b) => Query::starts_with(&artifact.get_id(), b.as_str()),
                    None => false,
                }
            }
            None => false,
        }
    }
    fn contains_title(user_input: &Artifact, artifact_title: &str) -> bool {
        trace!("Artifact: {:?} - Title: {}", user_input, artifact_title);
        user_input.get_title().contains(&artifact_title)
    }

    fn starts_with(artifact_id: &str, user_identfifier: &str) -> bool {
        trace!("Artifact: {:?} - Title: {}", user_identfifier, artifact_id);
        artifact_id.starts_with(&user_identfifier)
    }

    pub async fn fulfill_constraint(
        &mut self,
        constraint: Constraint,
    ) -> Result<Constraint, Error> {
        info!("fulfill_contsraint");
        match constraint {
            Constraint::List(mut con) => match self.find_board_id(&con.board.title).await {
                Ok(id) => {
                    con.board._id = id;
                    Ok(Constraint::List(con.clone()))
                }
                Err(_e) => Err(CliError::new_msg("Board not found").as_enum()),
            },
            Constraint::Card(mut con) => match self.find_board_id(&con.board.title).await {
                Ok(b_id) => match self.find_list_id(&b_id, &con.list.title).await {
                    Ok(l_id) => {
                        con.board._id = b_id;
                        con.list._id = l_id;
                        Ok(Constraint::Card(con.clone()))
                    }
                    Err(e) => {
                        trace!("{:?}", e);
                        Err(CliError::new_msg("List not found").as_enum())
                    }
                },
                Err(e) => {
                    trace!("{:?}", e);
                    Err(CliError::new_msg("Board not found").as_enum())
                }
            },
            Constraint::Checklist(mut con) => match self.find_board_id(&con.board.title).await {
                Ok(b_id) => match self.find_list_id(&b_id, &con.list.title).await {
                    Ok(l_id) => match self.find_card_id(&b_id, &l_id, &con.card.title).await {
                        Ok(c_id) => {
                            con.board._id = b_id;
                            con.card._id = c_id;
                            Ok(Constraint::Checklist(con.clone()))
                        }
                        Err(e) => {
                            trace!("{:?}", e);
                            Err(CliError::new_msg("Card not found").as_enum())
                        }
                    },
                    Err(e) => {
                        trace!("{:?}", e);
                        Err(CliError::new_msg("List not found").as_enum())
                    }
                },
                Err(e) => {
                    trace!("{:?}", e);
                    Err(CliError::new_msg("Board not found").as_enum())
                }
            },
            Constraint::Board(con) => Ok(Constraint::Board(con)),
            _ => Err(CliError::new_msg("NOT IMPLEMENTED").as_enum()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::mocks::{Mock, Mocks};
    #[tokio::test]
    async fn find_board_id() {
        #[cfg(not(feature = "store"))]
        let mut query = Query {
            config: UserConfig::mock(),
            filter: &String::new(),
        };
        #[cfg(feature = "store")]
        let mut query = Query {
            config: UserConfig::mock(),
            filter: &String::new(),
            deny_store_usage: true,
        };
        let res = query.find_board_id("fake-board-title-1").await.unwrap();
        assert_eq!(res, "fake-board-id-1");
    }

    #[tokio::test]
    async fn find_list_id() {
        #[cfg(not(feature = "store"))]
        let mut query = Query {
            config: UserConfig::mock(),
            filter: &String::new(),
        };
        #[cfg(feature = "store")]
        let mut query = Query {
            config: UserConfig::mock(),
            filter: &String::new(),
            deny_store_usage: true,
        };
        let res = query
            .find_list_id("fake-board-id-1", "fake-list-title-1")
            .await
            .unwrap();
        assert_eq!(res, "fake-list-id-1");
    }

    #[tokio::test]
    async fn find_card_id() {
        #[cfg(not(feature = "store"))]
        let mut query = Query {
            config: UserConfig::mock(),
            filter: &String::new(),
        };
        #[cfg(feature = "store")]
        let mut query = Query {
            config: UserConfig::mock(),
            filter: &String::new(),
            deny_store_usage: true,
        };

        let res = query
            .find_card_id("fake-board-id-2", "fake-list-id-1", "fake-card-title-1")
            .await
            .unwrap();
        assert_eq!(res, "fake-card-id-1");
    }

    #[tokio::test]
    async fn find_card_id_with_order() {
        #[cfg(not(feature = "store"))]
        let mut query = Query {
            config: UserConfig::mock(),
            filter: &String::from("b:f"),
        };
        #[cfg(feature = "store")]
        let mut query = Query {
            config: UserConfig::mock(),
            filter: &String::from("b:f"),
            deny_store_usage: true,
        };
        let res = query
            .find_card_id("fake-board-id-2", "fake-list-id-1", "fake-card-title-1")
            .await
            .unwrap();
        assert_eq!(res, "fake-card-id-1");
    }
    #[tokio::test]
    async fn request_boards() {
        #[cfg(not(feature = "store"))]
        let query = Query {
            config: UserConfig::mock(),
            filter: &String::new(),
        };
        #[cfg(feature = "store")]
        let query = Query {
            config: UserConfig::mock(),
            filter: &String::new(),
            deny_store_usage: false,
        };
        let res = query.request_boards().await.unwrap();
        assert_eq!(res, Vec::mocks(AType::Board));
    }

    #[tokio::test]
    async fn request_lists() {
        #[cfg(not(feature = "store"))]
        let query = Query {
            config: UserConfig::mock(),
            filter: &String::new(),
        };
        #[cfg(feature = "store")]
        let query = Query {
            config: UserConfig::mock(),
            filter: &String::new(),
            deny_store_usage: false,
        };
        let res = query.request_lists("fake-id-2").await.unwrap();
        assert_eq!(res, Vec::mocks(AType::List));
    }

    #[tokio::test]
    async fn request_cards() {
        #[cfg(not(feature = "store"))]
        let query = Query {
            config: UserConfig::mock(),
            filter: &String::new(),
        };
        #[cfg(feature = "store")]
        let query = Query {
            config: UserConfig::mock(),
            filter: &String::new(),
            deny_store_usage: false,
        };
        let res = query.request_cards("fake-id-2", "fake-id-2").await.unwrap();
        assert_eq!(res, Vec::mocks(AType::Card));
    }

    #[tokio::test]
    async fn request_swimlanes() {
        #[cfg(not(feature = "store"))]
        let query = Query {
            config: UserConfig::mock(),
            filter: &String::new(),
        };
        #[cfg(feature = "store")]
        let query = Query {
            config: UserConfig::mock(),
            filter: &String::new(),
            deny_store_usage: false,
        };
        let res = query.request_swimlanes("fake-id-2").await.unwrap();
        assert_eq!(res, Vec::mocks(AType::Swimlane));
    }

    #[tokio::test]
    async fn inquire() {
        #[cfg(not(feature = "store"))]
        let query = Query {
            config: UserConfig::mock(),
            filter: &String::new(),
        };
        #[cfg(feature = "store")]
        let query = Query {
            config: UserConfig::mock(),
            filter: &String::new(),
            deny_store_usage: false,
        };
        let mut res = query
            .inquire(AType::Board, None, None, false)
            .await
            .unwrap();
        assert_eq!(res.get_title(), "s");
        #[cfg(not(feature = "store"))]
        assert_eq!(res.remove(0).get_id(), "fake-board-id-1");
        #[cfg(feature = "store")]
        assert_eq!(res.remove(0).get_id(), "store-fake-board-id-1");
    }

    #[cfg(feature = "store")]
    #[cfg(test)]
    mod store_tests {
        use super::*;

        #[tokio::test]
        async fn find_board_id_with_store() {
            let mut query = Query {
                config: UserConfig::mock(),
                filter: &String::new(),
                deny_store_usage: false,
            };
            let res = query.find_board_id("fake-board-title-1").await.unwrap();
            #[cfg(not(feature = "store"))]
            assert_eq!(res, "fake-board-id-1");
            #[cfg(feature = "store")]
            assert_eq!(res, "store-fake-board-id-1");
        }

        #[tokio::test]
        async fn find_list_id_with_store() {
            let mut query = Query {
                config: UserConfig::mock(),
                filter: &String::new(),
                deny_store_usage: false,
            };
            let res = query
                .find_list_id("store-fake-board-id-1", "fake-list-title-1")
                .await
                .unwrap();
            #[cfg(not(feature = "store"))]
            assert_eq!(res, "fake-list-id-1");
            #[cfg(feature = "store")]
            assert_eq!(res, "store-fake-list-id-1");
        }

        #[tokio::test]
        async fn find_card_id_with_store_request_again() {
            let mut query = Query {
                config: UserConfig::mock(),
                filter: &String::new(),
                deny_store_usage: false,
            };
            let res = query
                .find_card_id("fake-id-board-2", "fake-id-card-1", "fake-card-title-1")
                .await
                .unwrap();
            #[cfg(not(feature = "store"))]
            assert_eq!(res, "fake-card-id-1");
            #[cfg(feature = "store")]
            assert_eq!(res, "store-fake-card-id-1");
        }
        #[tokio::test]
        async fn find_card_id_with_store() {
            let mut query = Query {
                config: UserConfig::mock(),
                filter: &String::new(),
                deny_store_usage: false,
            };
            let res = query
                .find_card_id(
                    "fake-id-board-2",
                    "fake-id-card-1",
                    "store-fake-card-title-1",
                )
                .await
                .unwrap();
            #[cfg(not(feature = "store"))]
            assert_eq!(res, "store-fake-card-id-1");

            #[cfg(feature = "store")]
            assert_eq!(res, "store-fake-card-id-1");
        }

        #[tokio::test]
        async fn find_card_id_with_store_with_order() {
            let mut query = Query {
                config: UserConfig::mock(),
                filter: &String::from("b:1"),
                deny_store_usage: false,
            };
            let res = query
                .find_card_id("fake-id-board-2", "fake-id-list-1", "fake-card-title-1")
                .await
                .unwrap();
            #[cfg(not(feature = "store"))]
            assert_eq!(res, "store-fake-card-id-1");
            #[cfg(feature = "store")]
            assert_eq!(res, "store-fake-card-id-1");
        }

        #[tokio::test]
        async fn request_boards_without_store() {
            let query = Query {
                config: UserConfig::mock(),
                filter: &String::new(),
                deny_store_usage: true,
            };
            let res = query.request_boards().await.unwrap();
            assert_eq!(res, Vec::mocks(AType::Board));
        }

        #[tokio::test]
        async fn request_lists_with_store() {
            let query = Query {
                config: UserConfig::mock(),
                filter: &String::new(),
                deny_store_usage: true,
            };
            let res = query.request_lists("fake-id-2").await.unwrap();
            assert_eq!(res, Vec::mocks(AType::List));
        }

        #[tokio::test]
        async fn request_card_with_store() {
            let query = Query {
                config: UserConfig::mock(),
                filter: &String::new(),
                deny_store_usage: true,
            };
            let res = query.request_cards("fake-id-2", "fake-id-2").await.unwrap();
            assert_eq!(res, Vec::mocks(AType::Card));
        }

        #[tokio::test]
        async fn request_swimlanes_with_store() {
            let query = Query {
                config: UserConfig::mock(),
                filter: &String::new(),
                deny_store_usage: true,
            };
            let res = query.request_swimlanes("fake-id-2").await.unwrap();
            assert_eq!(res, Vec::mocks(AType::Swimlane));
        }
    }
}
