use crate::{
    command::{Args as RArgs, ArtifactCommand},
    display::CliDisplay,
    error::kind::{CliError, Error, Transform},
    list::argument::Args,
    resolver::Query,
    result::kind::WekanResult,
    subcommand::CommonCommand as Command,
};
use log::{debug, info, trace};
use wekan_common::{
    artifact::{
        common::{AType, Artifact, Base, IdReturner},
        list::Details,
    },
    http::artifact::{CreateArtifact, ResponseOk},
    validation::constraint::ListConstraint as LConstraint,
};

use wekan_core::{client::Client, error::kind::Error as CoreError};

#[cfg(test)]
use crate::tests::mocks::{Artifacts, Operation};
#[cfg(not(test))]
use wekan_core::http::operation::{Artifacts, Operation};

pub struct Runner<'a> {
    pub args: Args,
    pub client: Client,
    pub constraint: LConstraint,
    pub format: String,
    pub display: CliDisplay,
    pub global_options: &'a RArgs,
}

impl<'a> ArtifactCommand<'a, Args, Client, LConstraint> for Runner<'a> {
    fn new(
        args: Args,
        client: Client,
        constraint: LConstraint,
        format: String,
        display: CliDisplay,
        global_options: &'a RArgs,
    ) -> Self {
        Self {
            args,
            client,
            constraint,
            format,
            display,
            global_options,
        }
    }
}

impl<'a> Runner<'a> {
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
                        let mut query = Query {
                            config: self.client.config.clone(),
                            deny_store_usage: self.global_options.no_store,
                        };
                        #[cfg(not(feature = "store"))]
                        let mut query = Query {
                            config: self.client.config.clone(),
                        };
                        let filter = match &self.global_options.filter {
                            Some(f) => f.to_owned(),
                            None => String::new(),
                        };
                        match query
                            .find_list_id(&self.constraint.board._id, n, &Some(filter))
                            .await
                        {
                            Ok(board_id) => {
                                match self.client.delete::<ResponseOk>(&board_id).await {
                                    Ok(_o) => WekanResult::new_msg("Successfully deleted").ok(),
                                    Err(e) => {
                                        trace!("{:?}", e);
                                        CliError::new_msg("Failed to delete").err()
                                    }
                                }
                            }
                            Err(_e) => Err(CliError::new_msg("List name does not exist").as_enum()),
                        }
                    }
                    None => Err(CliError::new_msg("List name not supplied").as_enum()),
                },
                Command::Inspect(i) => match &i.delegate.board_id {
                    Some(_id) => self.run_inspect(&i.id.to_owned()).await,
                    None => WekanResult::new_msg("Board id needs to be supplied.").ok(),
                },
                Command::Details(_d) => match self.args.name.to_owned() {
                    Some(n) => self.get_lists_or_details(&n).await,
                    None => WekanResult::new_msg("Board name needs to be supplied").ok(),
                },
            },
            None => WekanResult::new_workflow("Nothing selected", "Run 'list --help'").ok(),
        }
    }

    async fn print_requested_lists(&mut self) -> Result<WekanResult, Error> {
        info!("print_requested_lists");
        let mut client = self.client.clone();
        debug!("{:?}", client);
        let lists: Result<Vec<Artifact>, CoreError> = client.get_all(AType::Card).await;
        let results: Vec<Artifact> = match lists {
            Ok(res) => res,
            Err(_e) => Vec::<Artifact>::new(),
        };
        self.display
            .print_artifacts(results, Some(self.format.to_owned()))
    }

    async fn create_list(&self, card_title: String) -> Result<WekanResult, Error> {
        let mut client = self.client.clone();
        let c_a = CreateArtifact { title: card_title };
        match client.create::<CreateArtifact, ResponseOk>(&c_a).await {
            Ok(ok) => {
                trace!("{:?}", ok);
                WekanResult::new_msg("Successfully created").ok()
            }
            Err(e) => {
                debug!("{:?}", e);
                CliError::new_msg("Failed to create").err()
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
            self.client.to_owned().get_all(AType::List).await;
        let results: Vec<Artifact> = match lists {
            Ok(res) => res,
            Err(_e) => Vec::<Artifact>::new(),
        };
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
            Ok(o) => {
                self.get_cards_by_list_id(&o, &self.constraint.board._id.to_owned(), list_id)
                    .await
            }
            Err(e) => Err(e),
        }
    }
    async fn get_cards_by_list_id(
        &mut self,
        o: &WekanResult,
        board_id: &str,
        list_id: &str,
    ) -> Result<WekanResult, Error> {
        info!("get_cards");
        #[cfg(feature = "store")]
        let query = Query {
            config: self.client.config.clone(),
            deny_store_usage: self.global_options.no_store,
        };
        #[cfg(not(feature = "store"))]
        let query = Query {
            config: self.client.config.clone(),
        };
        match query
            .inquire(AType::Card, Some(board_id), Some(list_id), false)
            .await
        {
            Ok(cards) => {
                trace!("{:?}", cards);
                if !cards.is_empty() {
                    self.display.prepare_output(
                        &(o.get_msg() + "Following cards are available:\n"),
                        cards,
                        None,
                    )
                } else {
                    WekanResult::new_workflow(
                        &(o.get_msg() + "This list contains no card"),
                        "Create a card with subcommand 'card create --help'",
                    )
                    .ok()
                }
            }
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        subcommand::{Create, Details as SDetails, Remove},
        tests::mocks::Mock,
    };
    #[tokio::test]
    async fn run_no_options_specified() {
        let r_args = RArgs::mock();
        let mut runner = Runner::new(
            Args::mock(
                Some(String::from("fake-list-title-2")),
                String::from("fake-board-title-1"),
                None,
            ),
            Client::mock(),
            LConstraint {
                board: Artifact {
                    _id: String::from("fake-board-id-1"),
                    title: String::from("fake-board-title-1"),
                    r#type: AType::Board,
                },
            },
            String::new(),
            CliDisplay::new(Vec::new()),
            &r_args,
        );
        let res = runner.apply().await.unwrap();
        assert_eq!(res.get_msg(), "Nothing selected");
    }

    #[tokio::test]
    async fn run_no_options_details() {
        let r_args = RArgs::mock();
        let mut runner = Runner::new(
            Args::mock(
                Some(String::from("fake-list-title-2")),
                String::from("fake-board-title-2"),
                Some(Command::Details(SDetails {})),
            ),
            Client::mock(),
            LConstraint {
                board: Artifact {
                    _id: String::from("fake-board-id-2"),
                    title: String::from("fake-board-title-2"),
                    r#type: AType::Board,
                },
            },
            String::new(),
            CliDisplay::new(Vec::new()),
            &r_args,
        );
        let res = runner.apply().await.unwrap();
        #[cfg(not(feature = "store"))]
        let expected = concat!(
            "ID                TITLE             MODIFIED_AT       CREATED_AT\n",
            "my-f              fake-list-title   2020-10-12        2020-10-12\n----\n",
            "Following cards are available:\nID    TITLE\nfake  fake-card-title-1\n",
            "fake  fake-card-title-2\n\n----"
        );
        #[cfg(feature = "store")]
        let expected = concat!(
            "ID                TITLE             MODIFIED_AT       CREATED_AT\n",
            "my-f              fake-list-title   2020-10-12        2020-10-12\n----\n",
            "Following cards are available:\nID    TITLE\nstor  store-fake-card-title-1\n",
            "stor  store-fake-card-title-2\n\n----"
        );
        assert_eq!(res.get_msg(), expected);
    }

    #[tokio::test]
    async fn run_no_options_create() {
        let r_args = RArgs::mock();
        let mut runner = Runner::new(
            Args::mock(
                Some(String::from("fake-list-title-2")),
                String::from("fake-board-title-1"),
                Some(Command::Create(Create {
                    title: String::from("new-board"),
                })),
            ),
            Client::mock(),
            LConstraint {
                board: Artifact {
                    _id: String::from("fake-board-id-1"),
                    title: String::from("fake-board-title-1"),
                    r#type: AType::Board,
                },
            },
            String::new(),
            CliDisplay::new(Vec::new()),
            &r_args,
        );
        let res = runner.apply().await.unwrap();
        assert_eq!(res.get_msg(), "Successfully created");
    }

    #[tokio::test]
    async fn run_no_options_remove() {
        let r_args = RArgs::mock();
        let mut runner = Runner::new(
            Args::mock(
                Some(String::from("fake-list-title-2")),
                String::from("fake-board-title-1"),
                Some(Command::Remove(Remove {})),
            ),
            Client::mock(),
            LConstraint {
                board: Artifact {
                    _id: String::from("fake-board-id-1"),
                    title: String::from("fake-board-title-1"),
                    r#type: AType::Board,
                },
            },
            String::new(),
            CliDisplay::new(Vec::new()),
            &r_args,
        );
        let res = runner.apply().await.unwrap();
        assert_eq!(res.get_msg(), "Successfully deleted");
    }

    #[tokio::test]
    async fn run_with_special_output() {
        #[cfg(feature = "store")]
        let r_args = RArgs::mock_with(false, false, "long", "b:5");
        #[cfg(not(feature = "store"))]
        let r_args = RArgs::mock_with(false, "long", "b:5");

        let mut runner = Runner::new(
            Args::mock(
                Some(String::from("fake-list-title-2")),
                String::from("fake-board-title-2"),
                None,
            ),
            Client::mock(),
            LConstraint {
                board: Artifact {
                    _id: String::from("fake-board-id-2"),
                    title: String::from("fake-board-title-2"),
                    r#type: AType::Board,
                },
            },
            String::from("long"),
            CliDisplay::new(Vec::new()),
            &r_args,
        );
        let res = runner.apply().await.unwrap();
        assert_eq!(res.get_msg(), "Nothing selected");
    }
}
