#[cfg(feature = "store")]
use chrono::{prelude::*, DateTime};
use log::{debug, error, info, trace};
use regex::Regex;
use wekan_common::{
    artifact::common::{AType, Artifact, Base, QueryTrait, SortedArtifact},
    validation::authentication::TokenHeader,
};
use wekan_core::{
    client::{BoardApi, CardApi, Client, ListApi, SwimlaneApi},
    config::UserConfig,
    http::operation::Artifacts,
};

#[cfg(feature = "store")]
use crate::persistence::store::Store;

use crate::error::kind::{CliError, Error, Transform};

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
        let cards = match self.request_artifacts(AType::Card, board_id, list_id).await {
            Ok(o) => Ok(o),
            Err(e) => Err(e),
        };
        self.valid_response(cards, name, order).await
    }

    pub async fn find_swimlane_id(
        &mut self,
        board_id: &str,
        order: &Option<String>,
    ) -> Result<String, Error> {
        info!("find_swimlane_id");
        let swimlane = match self
            .request_artifacts(AType::Swimlane, board_id, &String::new())
            .await
        {
            Ok(o) => Ok(o),
            Err(e) => Err(e),
        };
        self.valid_response(swimlane, &String::from("Default"), order)
            .await
    }
    pub async fn find_list_id(
        &mut self,
        board_id: &str,
        name: &str,
        order: &Option<String>,
    ) -> Result<String, Error> {
        info!("find_list_id");
        let boards = match self
            .request_artifacts(AType::List, board_id, &String::new())
            .await
        {
            Ok(o) => Ok(o),
            Err(e) => Err(e),
        };
        self.valid_response(boards, name, order).await
    }
    pub async fn find_board_id(
        &mut self,
        name: &str,
        order: &Option<String>,
    ) -> Result<String, Error> {
        info!("find_board_id");
        let boards = match self
            .request_artifacts(AType::Board, &String::new(), &String::new())
            .await
        {
            Ok(o) => Ok(o),
            Err(e) => Err(e),
        };
        self.valid_response(boards, name, order).await
    }

    #[cfg(not(feature = "store"))]
    pub async fn request_artifacts(
        &self,
        artifact_variant: AType,
        board_id: &str,
        list_id: &str,
    ) -> Result<Vec<Artifact>, Error> {
        self.match_request_to_be_fullfilled(artifact_variant, board_id, list_id)
            .await
    }
    #[cfg(feature = "store")]
    pub async fn request_artifacts(
        &self,
        artifact_variant: AType,
        board_id: &str,
        list_id: &str,
    ) -> Result<Vec<Artifact>, Error> {
        if self.deny_store_usage {
            info!("Store disabled");
            self.match_request_to_be_fullfilled(artifact_variant, board_id, list_id)
                .await
        } else {
            match <Self as Store>::request_artifacts(
                self,
                artifact_variant.clone(),
                &(board_id.to_owned() + list_id),
            )
            .await
            {
                Ok(o) => match o.age.parse::<DateTime<Utc>>() {
                    Ok(t) => {
                        trace!("{:?}", o.payload);
                        match artifact_variant {
                            AType::Board => {
                                if Utc::now().hour() > t.hour() + 1 {
                                    debug!("New request");
                                    self.match_request_to_be_fullfilled(
                                        artifact_variant,
                                        board_id,
                                        list_id,
                                    )
                                    .await
                                } else {
                                    debug!("Take store");
                                    Ok(o.payload)
                                }
                            }
                            AType::List => {
                                if t.minute() + 15 > 60 || Utc::now().minute() > t.minute() + 15 {
                                    debug!("New request");
                                    self.match_request_to_be_fullfilled(
                                        artifact_variant,
                                        board_id,
                                        list_id,
                                    )
                                    .await
                                } else {
                                    debug!("Take store");
                                    Ok(o.payload)
                                }
                            }
                            AType::Card => {
                                if t.minute() + 5 > 60 || Utc::now().minute() > t.minute() + 5 {
                                    debug!("New request");
                                    self.match_request_to_be_fullfilled(
                                        artifact_variant,
                                        board_id,
                                        list_id,
                                    )
                                    .await
                                } else {
                                    debug!("Take store");
                                    Ok(o.payload)
                                }
                            }
                            AType::Swimlane => {
                                if t.minute() + 15 > 60 || Utc::now().minute() > t.minute() + 20 {
                                    debug!("New request");
                                    self.match_request_to_be_fullfilled(
                                        artifact_variant,
                                        board_id,
                                        list_id,
                                    )
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
                        self.match_request_to_be_fullfilled(artifact_variant, board_id, list_id)
                            .await
                    }
                },
                Err(_e) => {
                    self.match_request_to_be_fullfilled(artifact_variant, board_id, list_id)
                        .await
                }
            }
        }
    }

    async fn match_request_to_be_fullfilled(
        &self,
        artifact_variant: AType,
        board_id: &str,
        list_id: &str,
    ) -> Result<Vec<Artifact>, Error> {
        trace!("{:?}", artifact_variant);
        match artifact_variant {
            AType::Board => self.request_boards().await,
            AType::List => self.request_lists(board_id).await,
            AType::Card => self.request_cards(board_id, list_id).await,
            AType::Swimlane => self.request_swimlanes(board_id).await,
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

    async fn valid_response(
        &mut self,
        vecs: Result<Vec<impl QueryTrait>, Error>,
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
        vecs: Vec<impl QueryTrait>,
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
                debug!(
                    "Order is selected, Trying to filter based on the list given: {:?}",
                    order
                );

                if Query::match_title_by_type(artifact, s) {
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
}
