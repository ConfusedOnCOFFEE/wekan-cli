#[cfg(feature = "store")]
use chrono::{prelude::*, DateTime};
use log::{debug, error, info, trace};
use regex::Regex;
use wekan_common::{
    artifact::common::{AType, Artifact, Base, IdReturner, SortedArtifact, WekanDisplay},
    validation::authentication::TokenHeader,
};
use wekan_core::{
    client::{BoardApi, CardApi, Client, ListApi, SwimlaneApi},
    config::UserConfig,
};

use crate::error::kind::{CliError, Error, Transform};
#[cfg(feature = "store")]
use crate::store::Store;
#[cfg(test)]
use crate::tests::mocks::{Artifacts, Mock};
#[cfg(not(test))]
use wekan_core::http::operation::Artifacts;

pub struct Query {
    pub config: UserConfig,
    #[cfg(feature = "store")]
    pub deny_store_usage: bool,
}

impl Query {
    pub async fn find_card_id(
        &mut self,
        board_id: &str,
        list_id: &str,
        name: &str,
        order: &Option<String>,
    ) -> Result<String, Error> {
        info!("find_card_id");
        let cards = match self
            .inquire(AType::Card, Some(board_id), Some(list_id))
            .await
        {
            Ok(o) => Ok(o),
            Err(e) => {
                trace!("{:?}", e);
                self.fulfill_inquiry(AType::Card, Some(board_id), Some(list_id))
                    .await
            }
        };
        self.confirm_valid_name(cards, name, order).await
    }

    pub async fn find_swimlane_id(
        &mut self,
        board_id: &str,
        order: &Option<String>,
    ) -> Result<String, Error> {
        info!("find_swimlane_id");
        let swimlane = match self.inquire(AType::Swimlane, Some(board_id), None).await {
            Ok(o) => Ok(o),
            Err(e) => {
                trace!("{:?}", e);
                self.fulfill_inquiry(AType::Card, Some(board_id), None)
                    .await
            }
        };
        self.confirm_valid_name(swimlane, &String::from("Default"), order)
            .await
    }

    pub async fn find_list_id(
        &mut self,
        board_id: &str,
        name: &str,
        order: &Option<String>,
    ) -> Result<String, Error> {
        info!("find_list_id");
        let boards = match self.inquire(AType::List, Some(board_id), None).await {
            Ok(o) => Ok(o),
            Err(e) => {
                trace!("{:?}", e);
                self.fulfill_inquiry(AType::Card, Some(board_id), None)
                    .await
            }
        };
        self.confirm_valid_name(boards, name, order).await
    }

    pub async fn find_board_id(
        &mut self,
        name: &str,
        order: &Option<String>,
    ) -> Result<String, Error> {
        info!("find_board_id");
        let boards = match self.inquire(AType::Board, None, None).await {
            Ok(o) => Ok(o),
            Err(e) => {
                trace!("{:?}", e);
                self.fulfill_inquiry(AType::Card, None, None).await
            }
        };
        self.confirm_valid_name(boards, name, order).await
    }

    #[cfg(not(feature = "store"))]
    pub async fn inquire(
        &self,
        artifact_variant: AType,
        board_id: Option<&str>,
        list_id: Option<&str>,
    ) -> Result<Vec<Artifact>, Error> {
        self.fulfill_inquiry(artifact_variant, board_id, list_id)
            .await
    }

    #[cfg(feature = "store")]
    pub async fn inquire(
        &self,
        artifact_variant: AType,
        board_id: Option<&str>,
        list_id: Option<&str>,
    ) -> Result<Vec<Artifact>, Error> {
        if self.deny_store_usage {
            info!("Store disabled");
            self.fulfill_inquiry(artifact_variant, board_id, list_id)
                .await
        } else {
            let join_ids = match board_id {
                Some(b) => match list_id {
                    Some(l) => b.to_owned() + l,
                    None => b.to_string(),
                },
                None => String::new(),
            };
            match self
                .lookup_artifacts(artifact_variant.clone(), &join_ids)
                .await
            {
                Ok(o) => match o.age.parse::<DateTime<Utc>>() {
                    Ok(t) => {
                        trace!("{:?}", o.payload);
                        match artifact_variant {
                            AType::Board => {
                                if Utc::now().hour() > t.hour() + 1 {
                                    debug!("New request");
                                    self.fulfill_inquiry(artifact_variant, board_id, list_id)
                                        .await
                                } else {
                                    debug!("Take store");
                                    Ok(o.payload)
                                }
                            }
                            AType::List => {
                                if t.minute() + 5 > 60 || Utc::now().minute() > t.minute() + 15 {
                                    debug!("New request");
                                    self.fulfill_inquiry(artifact_variant, board_id, list_id)
                                        .await
                                } else {
                                    debug!("Take store");
                                    Ok(o.payload)
                                }
                            }
                            AType::Card => {
                                if t.minute() + 2 > 60 || Utc::now().minute() > t.minute() + 5 {
                                    debug!("New request");
                                    self.fulfill_inquiry(artifact_variant, board_id, list_id)
                                        .await
                                } else {
                                    debug!("Take store");
                                    Ok(o.payload)
                                }
                            }
                            AType::Swimlane => {
                                if t.minute() + 5 > 60 || Utc::now().minute() > t.minute() + 20 {
                                    debug!("New request");
                                    self.fulfill_inquiry(artifact_variant, board_id, list_id)
                                        .await
                                } else {
                                    debug!("Take store");
                                    Ok(o.payload)
                                }
                            }
                            _ => {
                                error!("Not a AType or empty list");
                                Ok(Vec::new())
                            }
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
    }

    async fn fulfill_inquiry(
        &self,
        artifact_variant: AType,
        board_id: Option<&str>,
        list_id: Option<&str>,
    ) -> Result<Vec<Artifact>, Error> {
        trace!("{:?}", artifact_variant);
        match artifact_variant {
            AType::Board => self.request_boards().await,
            AType::List | AType::Swimlane | AType::Card => match board_id {
                Some(b) => match artifact_variant {
                    AType::List => self.request_lists(b).await,
                    AType::Swimlane => self.request_swimlanes(b).await,
                    AType::Card => match list_id {
                        Some(l) => self.request_cards(b, l).await,
                        None => panic!("List id needs to be supplied"),
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
        let mut client = <Client as CardApi>::new(
            self.config.to_owned(),
            board_id.to_string(),
            list_id.to_string(),
        );
        match client.get_all(AType::Card).await {
            Ok(o) => Ok(o),
            Err(e) => Err(Error::from(e)),
        }
    }
    async fn request_lists(&self, board_id: &str) -> Result<Vec<Artifact>, Error> {
        info!("request_lists");
        let mut client = <Client as ListApi>::new(self.config.to_owned(), board_id.to_string());
        match client.get_all(AType::List).await {
            Ok(o) => Ok(o),
            Err(e) => Err(Error::from(e)),
        }
    }

    async fn confirm_valid_name(
        &mut self,
        vecs: Result<Vec<impl WekanDisplay>, Error>,
        name: &str,
        order: &Option<String>,
    ) -> Result<String, Error> {
        match vecs {
            Ok(r#as) => self.extract_id(r#as, name, order).await,
            Err(_e) => Err(CliError::new_msg("No artifacts have been fonud.").as_enum()),
        }
    }

    async fn extract_id(
        &mut self,
        vecs: Vec<impl WekanDisplay>,
        name: &str,
        order: &Option<String>,
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
                    debug!("extract_id");
                    trace!("{:?} and the query name: {}", a, name);
                    if Query::check_artifact(&a, name, order) {
                        debug!("One artifact found.");
                        break Ok(a.get_id());
                    }
                }
                None => break Err(CliError::new_msg("Artifact not found.").as_enum()),
            };
        }
    }

    fn check_artifact(artifact: &Artifact, name: &str, order: &Option<String>) -> bool {
        match order {
            Some(ref s) => {
                if !s.is_empty() && Query::match_title_by_type(artifact, s) {
                    true
                } else {
                    Query::contains_title(artifact, name)
                }
            }
            None => Query::contains_title(artifact, name),
        }
    }

    fn match_title_by_type(artifact: &Artifact, filter: &str) -> bool {
        match artifact.get_type() {
            AType::Board => {
                let b = Regex::new(r"b:(?P<board>[a-z0-9A-Z]{1,4}),l:.*").unwrap();
                let before = "b:a,l:Ej,c:I";
                let after = b.captures(before).unwrap();
                assert_eq!(after.name("board").unwrap().as_str(), "a");
                let before = "b:a8as,l:Eklk,c:I09";
                let after = b.captures(before).unwrap();
                assert_eq!(after.name("board").unwrap().as_str(), "a8as");
                trace!("Board: {:?} vs {:?}", filter, artifact.get_id());
                Query::match_captured_user_input(b, filter, artifact)
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
                Query::match_captured_user_input(b, filter, artifact)
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
                Query::match_captured_user_input(b, filter, artifact)
            }
            _ => false,
        }
    }

    fn match_captured_user_input(regex: Regex, filter: &str, artifact: &Artifact) -> bool {
        trace!("Card: {:?} vs {:?}", filter, artifact);
        match regex.captures(filter) {
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
        debug!("Artifact: {:?} - Title: {:?}", user_input, artifact_title);
        user_input.get_title().contains(&artifact_title)
    }

    fn starts_with(artifact_id: &str, user_identfifier: &str) -> bool {
        debug!(
            "Artifact: {:?} - Title: {:?}",
            user_identfifier, artifact_id
        );
        artifact_id.starts_with(&user_identfifier)
    }

    #[cfg(test)]
    #[cfg(feature = "store")]
    fn mock(store: bool) -> Self {
        Query {
            config: UserConfig::mock(),
            deny_store_usage: store,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::mocks::Mocks;
    #[cfg(not(feature = "store"))]
    impl Mock for Query {
        fn mock() -> Self {
            Query {
                config: UserConfig::mock(),
            }
        }
    }

    #[tokio::test]
    async fn find_board_id() {
        #[cfg(not(feature = "store"))]
        let mut query = Query::mock();
        #[cfg(feature = "store")]
        let mut query = Query::mock(true);
        let res = query
            .find_board_id("fake-board-title-1", &None)
            .await
            .unwrap();
        assert_eq!(res, "fake-board-id-1");
    }

    #[tokio::test]
    async fn find_list_id() {
        #[cfg(not(feature = "store"))]
        let mut query = Query::mock();
        #[cfg(feature = "store")]
        let mut query = Query::mock(true);
        let res = query
            .find_list_id("fake-board-id-1", "fake-list-title-1", &None)
            .await
            .unwrap();
        assert_eq!(res, "fake-list-id-1");
    }

    #[tokio::test]
    async fn find_card_id() {
        #[cfg(not(feature = "store"))]
        let mut query = Query::mock();
        #[cfg(feature = "store")]
        let mut query = Query::mock(true);
        let res = query
            .find_card_id(
                "fake-board-id-2",
                "fake-list-id-1",
                "fake-card-title-1",
                &None,
            )
            .await
            .unwrap();
        assert_eq!(res, "fake-card-id-1");
    }

    #[tokio::test]
    async fn find_card_id_with_order() {
        #[cfg(not(feature = "store"))]
        let mut query = Query::mock();
        #[cfg(feature = "store")]
        let mut query = Query::mock(true);
        let res = query
            .find_card_id(
                "fake-board-id-2",
                "fake-list-id-1",
                "fake-card-title-1",
                &Some(String::from("b:f")),
            )
            .await
            .unwrap();
        assert_eq!(res, "fake-card-id-1");
    }
    #[tokio::test]
    async fn request_boards() {
        #[cfg(not(feature = "store"))]
        let query = Query::mock();
        #[cfg(feature = "store")]
        let query = Query::mock(false);
        let res = query.request_boards().await.unwrap();
        assert_eq!(res, Vec::mocks(AType::Board));
    }

    #[tokio::test]
    async fn request_lists() {
        #[cfg(not(feature = "store"))]
        let query = Query::mock();
        #[cfg(feature = "store")]
        let query = Query::mock(false);
        let res = query.request_lists("fake-id-2").await.unwrap();
        assert_eq!(res, Vec::mocks(AType::List));
    }

    #[tokio::test]
    async fn request_cards() {
        #[cfg(not(feature = "store"))]
        let query = Query::mock();
        #[cfg(feature = "store")]
        let query = Query::mock(false);
        let res = query.request_cards("fake-id-2", "fake-id-2").await.unwrap();
        assert_eq!(res, Vec::mocks(AType::Card));
    }

    #[tokio::test]
    async fn request_swimlanes() {
        #[cfg(not(feature = "store"))]
        let query = Query::mock();
        #[cfg(feature = "store")]
        let query = Query::mock(false);
        let res = query.request_swimlanes("fake-id-2").await.unwrap();
        assert_eq!(res, Vec::mocks(AType::Swimlane));
    }

    #[tokio::test]
    async fn inquire() {
        #[cfg(not(feature = "store"))]
        let query = Query::mock();
        #[cfg(feature = "store")]
        let query = Query::mock(false);
        let mut res = query.inquire(AType::Board, None, None).await.unwrap();
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
            let mut query = Query::mock(false);
            let res = query
                .find_board_id("fake-board-title-1", &None)
                .await
                .unwrap();
            #[cfg(not(feature = "store"))]
            assert_eq!(res, "fake-board-id-1");
            #[cfg(feature = "store")]
            assert_eq!(res, "store-fake-board-id-1");
        }

        #[tokio::test]
        async fn find_list_id_with_store() {
            let mut query = Query::mock(false);
            let res = query
                .find_list_id("store-fake-board-id-1", "fake-list-title-1", &None)
                .await
                .unwrap();
            #[cfg(not(feature = "store"))]
            assert_eq!(res, "fake-list-id-1");
            #[cfg(feature = "store")]
            assert_eq!(res, "store-fake-list-id-1");
        }

        #[tokio::test]
        async fn find_card_id_with_store_request_again() {
            let mut query = Query::mock(false);
            let res = query
                .find_card_id(
                    "fake-id-board-2",
                    "fake-id-card-1",
                    "fake-card-title-1",
                    &None,
                )
                .await
                .unwrap();
            #[cfg(not(feature = "store"))]
            assert_eq!(res, "fake-card-id-1");
            #[cfg(feature = "store")]
            assert_eq!(res, "store-fake-card-id-1");
        }
        #[tokio::test]
        async fn find_card_id_with_store() {
            let mut query = Query::mock(false);
            let res = query
                .find_card_id(
                    "fake-id-board-2",
                    "fake-id-card-1",
                    "store-fake-card-title-1",
                    &None,
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
            let mut query = Query::mock(false);
            let res = query
                .find_card_id(
                    "fake-id-board-2",
                    "fake-id-list-1",
                    "fake-card-title-1",
                    &Some(String::from("b:1")),
                )
                .await
                .unwrap();
            #[cfg(not(feature = "store"))]
            assert_eq!(res, "store-fake-card-id-1");
            #[cfg(feature = "store")]
            assert_eq!(res, "store-fake-card-id-1");
        }

        #[tokio::test]
        async fn request_boards_with_store() {
            let query = Query::mock(true);
            let res = query.request_boards().await.unwrap();
            assert_eq!(res, Vec::mocks(AType::Board));
        }

        #[tokio::test]
        async fn request_lists_with_store() {
            let query = Query::mock(true);
            let res = query.request_lists("fake-id-2").await.unwrap();
            assert_eq!(res, Vec::mocks(AType::List));
        }

        #[tokio::test]
        async fn request_card_with_store() {
            let query = Query::mock(true);
            let res = query.request_cards("fake-id-2", "fake-id-2").await.unwrap();
            assert_eq!(res, Vec::mocks(AType::Card));
        }

        #[tokio::test]
        async fn request_swimlanes_with_store() {
            let query = Query::mock(true);
            let res = query.request_swimlanes("fake-id-2").await.unwrap();
            assert_eq!(res, Vec::mocks(AType::Swimlane));
        }
    }
}
