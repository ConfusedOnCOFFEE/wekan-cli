use crate::{
    command::{ArtifactCommand, WekanParser},
    display::CliDisplay,
    error::kind::{CliError, Error, Transform},
    list::argument::Args,
    resolver::Query,
    result::kind::WekanResult,
    subcommand::CommonCommand as Command,
};
use clap::Parser;
use log::{debug, info, trace};
use wekan_common::{
    artifact::{
        common::{AType, Artifact, Base, IdReturner},
        list::Details,
    },
    http::artifact::{CreateArtifact, ResponseOk},
    validation::constraint::ListConstraint as LConstraint,
};
use wekan_core_derive::Unwrapper as DeriveUnwrapper;

use wekan_core::{
    client::Client,
    error::kind::Error as CoreError,
    http::{
        operation::{Artifacts, Operation},
        util::Unwrapper,
    },
};

#[derive(DeriveUnwrapper)]
pub struct Runner {
    pub args: Args,
    pub client: Client,
    pub constraint: LConstraint,
    pub format: String,
    pub display: CliDisplay,
}

impl ArtifactCommand<Args, Client, LConstraint> for Runner {
    fn new(
        args: Args,
        client: Client,
        constraint: LConstraint,
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

impl Runner {
    pub async fn apply(&mut self) -> Result<WekanResult, Error> {
        info!("apply");
        self.run_subcommand().await
    }

    async fn run_subcommand(&mut self) -> Result<WekanResult, Error> {
        info!("run_subcommand");
        match self.args.command.to_owned() {
            Some(c) => match c {
                Command::Ls(_ls) => self.print_requested_lists().await,
                Command::Create(c) => self.create_list(c.title.to_owned()).await,
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
                        match query
                            .find_list_id(&self.constraint.board._id, n, &filter)
                            .await
                        {
                            Ok(board_id) => {
                                match self.client.delete::<ResponseOk>(&board_id).await {
                                    Ok(_o) => WekanResult::new_msg("Delete successfull.").ok(),
                                    Err(e) => {
                                        trace!("{:?}", e);
                                        CliError::new_msg("Deletion failed").err()
                                    }
                                }
                            }
                            Err(_e) => Err(CliError::new_msg("List name does not exist").as_enum()),
                        }
                    }
                    None => Err(CliError::new_msg("List name not supplied.").as_enum()),
                },
                Command::Inspect(i) => match &i.delegate.board_id {
                    Some(_id) => self.run_inspect(&i.id.to_owned()).await,
                    None => WekanResult::new_msg("Board id needs to be supplied.").ok(),
                },
                Command::Details(_d) => match self.args.name.to_owned() {
                    Some(n) => self.get_lists_or_details(&n).await,
                    None => WekanResult::new_msg("Board name needs to be supplied.").ok(),
                },
            },
            None => WekanResult::new_workflow("Nothing selected.", "Run 'list --help'").ok(),
        }
    }

    async fn print_requested_lists(&mut self) -> Result<WekanResult, Error> {
        info!("print_requested_lists");
        let mut client = self.client.clone();
        debug!("{:?}", client);
        let lists: Result<Vec<Artifact>, CoreError> = client.get_all(AType::Card).await;
        let results: Vec<Artifact> = Self::get_result(lists).await;
        self.display
            .print_artifacts(results, self.format.to_owned())
    }

    async fn create_list(&self, card_title: String) -> Result<WekanResult, Error> {
        let mut client = self.client.clone();
        let c_a = CreateArtifact { title: card_title };
        match client.create::<CreateArtifact, ResponseOk>(&c_a).await {
            Ok(ok) => {
                trace!("{:?}", ok);
                WekanResult::new_workflow(
                    "New List created.",
                    "See the details of a list or create a card for it with 'card -b <BOARD_NAME> -l <LIST_NAME> create [CARD_NAME] --description [CARD_DESCRIPTION]'").ok()
            }
            Err(e) => {
                debug!("{:?}", e);
                CliError::new_msg("Create list failed.").err()
            }
        }
    }

    async fn run_inspect(&mut self, list_id: &str) -> Result<WekanResult, Error> {
        info!("run_inspect");
        let mut client = self.client.clone();
        let list = client.get_one::<Details>(list_id).await.unwrap();
        self.display.print_details(list, Some("long".to_string()))
    }

    async fn get_lists_or_details(&mut self, list_name: &str) -> Result<WekanResult, Error> {
        info!("get_lists");
        let lists: Result<Vec<Artifact>, CoreError> =
            self.client.to_owned().get_all(AType::Card).await;
        let results: Vec<Artifact> = Self::get_result(lists).await;
        let mut iter = results.iter();
        trace!("Lists: {:?} - {}", results, list_name);
        loop {
            match iter.next() {
                Some(vec) => {
                    if vec.get_title().contains(list_name) {
                        trace!(
                            "Matched with board_id: {} and list_name: {}",
                            self.constraint.board._id,
                            list_name
                        );
                        break self.get_details(&vec.get_id()).await;
                    }
                }
                None => {
                    break WekanResult::new_workflow(
                        "List doesn't exist",
                        "Try with option s or run 'board --help'",
                    )
                    .ok()
                }
            }
        }
    }

    async fn get_details(&mut self, list_id: &str) -> Result<WekanResult, Error> {
        let mut client = self.client.clone();
        let list = client.get_one::<Details>(list_id).await.unwrap();
        match self
            .display
            .print_details(list, Some(self.format.to_owned()))
        {
            Ok(_o) => {
                self.get_cards_by_list_id(&self.constraint.board._id.to_owned(), list_id)
                    .await
            }
            Err(e) => Err(e),
        }
    }
    async fn get_cards_by_list_id(
        &mut self,
        board_id: &str,
        list_id: &str,
    ) -> Result<WekanResult, Error> {
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
            .inquire(AType::Card, board_id, list_id)
            .await
        {
            Ok(cards) => {
                trace!("{:?}", cards);
                if !cards.is_empty() {
                    println!("Following cards are available:");
                    self.display.print_artifacts(cards, String::from("long"))
                } else {
                    WekanResult::new_workflow(
                        "This list contains no card.",
                        "Create a card with 'card create --help'",
                    )
                    .ok()
                }
            }
            Err(e) => Err(e),
        }
    }
}
