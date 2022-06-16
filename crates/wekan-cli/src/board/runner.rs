use async_trait::async_trait;
use clap::Parser;
use log::{debug, info, trace};
use wekan_common::{
    artifact::board::Details,
    artifact::common::AType,
    http::{
        artifact::ResponseOk,
        board::{CreateBoard, CreatedBoard},
    },
    validation::{
        authentication::TokenHeader,
        constraint::{BoardConstraint as BConstraint, Constraint},
    },
};

use super::argument::Args;
use crate::{
    command::{ArtifactCommand, CommonRunsSimplified, RootCommand, WekanParser},
    display::CliDisplay,
    error::kind::{CliError, Error, Transform},
    resolver::Query,
    result::kind::WekanResult,
    subcommand::CommonCommand as Command,
};
#[cfg(test)]
use wekan_common::{
    artifact::common::Artifact,
    artifact::tests::{MockNewResponse, MockReturn},
    http::artifact::{Deleted, IdResponse, RequestBody},
};
#[cfg(test)]
use wekan_core::client::{BoardApi, Client};
#[cfg(not(test))]
use wekan_core::{
    client::{BoardApi, Client},
    http::operation::{Artifacts, Operation},
};

pub struct Runner {
    pub args: Args,
    pub client: Client,
    pub constraint: BConstraint,
    pub format: String,
    pub display: CliDisplay,
}

impl ArtifactCommand<Args, Client, BConstraint> for Runner {
    fn new(
        args: Args,
        client: Client,
        constraint: BConstraint,
        format: String,
        display: CliDisplay,
    ) -> Self {
        Self {
            args,
            client,
            constraint,
            format,
            display,
        }
    }
}

#[async_trait]
impl RootCommand for Runner {
    async fn run(&mut self) -> Result<WekanResult, Error> {
        debug!("run");
        match self.args.command.clone() {
            Some(cmd) => self.run_subcommand(cmd).await,
            None => match self.check_supplied_name() {
                Ok(n) => self.use_rootcommand(&n).await,
                Err(e) => Err(e),
            },
        }
    }
    async fn use_rootcommand(&mut self, name: &str) -> Result<WekanResult, Error> {
        let parser = WekanParser::parse();
        #[cfg(feature = "store")]
        let mut query = Query {
            config: self.client.config.clone(),
            deny_store_usage: parser.delegate.no_store,
        };
        #[cfg(not(feature = "store"))]
        let mut query = Query {
            config: self.client.config.clone(),
        };
        match query
            .find_board_id(name, &parser.delegate.filter.to_owned())
            .await
        {
            Ok(id) => {
                let board = self.client.get_one::<Details>(&id).await.unwrap();
                match self
                    .display
                    .print_details(board, parser.delegate.filter.to_owned())
                {
                    Ok(_o) => self.get_list_by_board_id(&id).await,
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(e),
        }
    }
}

#[async_trait]
impl CommonRunsSimplified for Runner {
    async fn list(&mut self) -> Result<WekanResult, Error> {
        self.client
            .set_base(&("users/".to_owned() + &self.client.get_user_id() + "/boards"));
        match self.client.get_all(AType::Board).await {
            Ok(ok) => {
                debug!("{:?}", ok);
                self.display.print_artifacts(ok, self.format.to_owned())
            }
            Err(e) => Err(Error::Core(e)),
        }
    }
    async fn details(&mut self, name: Option<String>) -> Result<WekanResult, Error> {
        match &name {
            Some(n) => {
                #[cfg(feature = "store")]
                let parser = WekanParser::parse();
                #[cfg(feature = "store")]
                let mut query = Query {
                    config: self.client.config.clone(),
                    deny_store_usage: parser.delegate.no_store,
                };
                #[cfg(not(feature = "store"))]
                let mut query = Query {
                    config: self.client.config.clone(),
                };
                let filter = WekanParser::parse().delegate.filter;
                let format = WekanParser::parse().delegate.filter;
                match query.find_board_id(n, &filter).await {
                    Ok(board_id) => {
                        let board = self.client.get_one::<Details>(&board_id).await.unwrap();
                        self.display.print_details(board, format)
                    }
                    Err(_e) => Err(CliError::new_msg("Board name does not exist").as_enum()),
                }
            }
            None => Err(CliError::new_msg("Board name not supplied.").as_enum()),
        }
    }
}
impl Runner {
    async fn run_subcommand(&mut self, cmd: Command) -> Result<WekanResult, Error> {
        debug!("run_subcommand");
        let mut client = self.client.clone();
        client.set_base("boards/");
        match cmd {
            Command::Ls(_l) => self.list().await,
            Command::Create(c) => {
                trace!("{:?}", c);
                let body = CreateBoard {
                    title: c.title.to_string(),
                    owner: client.get_user_id(),
                    permission: Some(String::from("private")),
                    color: None,
                    is_admin: None,
                    is_active: None,
                    is_no_comments: None,
                    is_comment_only: None,
                    is_worker: None,
                };
                match client.create::<CreateBoard, CreatedBoard>(&body).await {
                    Ok(ok) => {
                        trace!("{:?}", ok);
                        WekanResult::new_workflow(
                            "New board created.",
                            "Create a list with 'list -b <BOARD_NAME> create [LIST_NAME]",
                        )
                        .ok()
                    }
                    Err(e) => {
                        debug!("{:?}", e);
                        CliError::new(
                            4,
                            "Create board failed. Did you login?",
                            Constraint::Login(true),
                        )
                        .err()
                    }
                }
            }
            Command::Remove(_r) => match &self.args.name {
                Some(n) => {
                    #[cfg(feature = "store")]
                    let parser = WekanParser::parse();
                    #[cfg(feature = "store")]
                    let mut query = Query {
                        config: self.client.config.clone(),
                        deny_store_usage: parser.delegate.no_store,
                    };
                    #[cfg(not(feature = "store"))]
                    let mut query = Query {
                        config: self.client.config.clone(),
                    };
                    let filter = WekanParser::parse().delegate.filter;
                    match query.find_board_id(n, &filter).await {
                        Ok(board_id) => match client.delete::<ResponseOk>(&board_id).await {
                            Ok(_o) => WekanResult::new_msg("Delete successfull.").ok(),
                            Err(e) => {
                                trace!("{:?}", e);
                                CliError::new_msg("Deletion failed").err()
                            }
                        },
                        Err(_e) => Err(CliError::new_msg("Board name does not exist").as_enum()),
                    }
                }
                None => Err(CliError::new_msg("Board name not supplied.").as_enum()),
            },
            Command::Inspect(i) => {
                let board = client.get_one::<Details>(&i.id).await.unwrap();
                self.display.print_details(board, Some("long".to_string()))
            }
            Command::Details(_d) => self.details(self.args.name.to_owned()).await,
        }
    }

    fn check_supplied_name(&self) -> Result<String, Error> {
        match &self.args.name {
            Some(n) => Ok(n.to_string()),
            None => Err(CliError::new_msg("No name supplied").as_enum()),
        }
    }
    async fn get_list_by_board_id(&mut self, board_id: &str) -> Result<WekanResult, Error> {
        info!("get_cards");
        #[cfg(feature = "store")]
        let parser = WekanParser::parse();
        #[cfg(feature = "store")]
        let query = Query {
            config: self.client.config.clone(),
            deny_store_usage: parser.delegate.no_store,
        };
        #[cfg(not(feature = "store"))]
        let query = Query {
            config: self.client.config.clone(),
        };
        match query
            .request_artifacts(AType::Board, board_id, &String::new())
            .await
        {
            Ok(lists) => {
                trace!("{:?}", lists);
                if !lists.is_empty() {
                    println!("Following lists are available:");
                    self.display.print_artifacts(lists, String::from("long"))
                } else {
                    WekanResult::new_workflow(
                        "This boards contains no lists.",
                        "Create a list with 'list -b <BOARD_NAME> create [CARD_NAME]'",
                    )
                    .ok()
                }
            }
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
#[async_trait]
pub trait Operation {
    async fn create<
        U: RequestBody,
        T: MockNewResponse + Send + std::fmt::Debug + serde::de::DeserializeOwned + 'static,
    >(
        &mut self,
        _body: &U,
    ) -> Result<T, wekan_core::error::kind::Error> {
        Ok(T::new())
    }
    async fn delete<T: Deleted + MockReturn + IdResponse>(
        &mut self,
        id: &str,
    ) -> Result<T, wekan_core::error::kind::Error> {
        Ok(T::success(Some(id.to_string())))
    }
    async fn put<U: RequestBody, T: MockReturn + IdResponse>(
        &mut self,
        id: &str,
        _body: &U,
    ) -> Result<T, wekan_core::error::kind::Error> {
        Ok(T::success(Some(id.to_string())))
    }
}
#[cfg(test)]
impl Operation for wekan_core::client::Client {}

#[cfg(test)]
#[async_trait]
pub trait Artifacts {
    async fn get_all(&mut self, t: AType) -> Result<Vec<Artifact>, wekan_core::error::kind::Error> {
        Ok(vec![
            Artifact::new("fake-id-1", "fake-title", t.clone()),
            Artifact::new("fake-id-2", "fake-title2", t),
        ])
    }
    async fn get_one<T: MockNewResponse + serde::de::DeserializeOwned + 'static>(
        &mut self,
        _id: &str,
    ) -> Result<T, wekan_core::error::kind::Error> {
        Ok(T::new())
    }
}
#[cfg(test)]
impl Artifacts for wekan_core::client::Client {}
