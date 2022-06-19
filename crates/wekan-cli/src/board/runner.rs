use async_trait::async_trait;
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
    command::{Args as RArgs, ArtifactCommand, CommonRunsSimplified, RootCommandRunner},
    display::CliDisplay,
    error::kind::{CliError, Error, Transform},
    resolver::Query,
    result::kind::WekanResult,
    subcommand::CommonCommand as Command,
};

#[cfg(test)]
use crate::tests::mocks::{Artifacts, Operation};
use wekan_core::client::{BoardApi, Client};
#[cfg(not(test))]
use wekan_core::http::operation::{Artifacts, Operation};

pub struct Runner<'a> {
    pub args: Args,
    pub client: Client,
    pub constraint: BConstraint,
    pub format: String,
    pub display: CliDisplay,
    pub global_options: &'a RArgs,
}

impl<'a> ArtifactCommand<'a, Args, Client, BConstraint> for Runner<'a> {
    fn new(
        args: Args,
        client: Client,
        constraint: BConstraint,
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

#[async_trait]
impl<'a> RootCommandRunner for Runner<'a> {
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
        #[cfg(feature = "store")]
        let mut query = Query {
            config: self.client.config.clone(),
            deny_store_usage: self.global_options.no_store,
        };
        #[cfg(not(feature = "store"))]
        let mut query = Query {
            config: self.client.config.clone(),
        };
        match query
            .find_board_id(name, &self.global_options.filter.to_owned())
            .await
        {
            Ok(id) => {
                let board = self.client.get_one::<Details>(&id).await.unwrap();
                match self
                    .display
                    .print_details(board, self.global_options.output_format.to_owned())
                {
                    Ok(o) => self.get_list_by_board_id(&o, &id).await,
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(e),
        }
    }
}

#[async_trait]
impl<'a> CommonRunsSimplified for Runner<'a> {
    async fn list(&mut self) -> Result<WekanResult, Error> {
        self.client
            .set_base(&("users/".to_owned() + &self.client.get_user_id() + "/boards"));
        match self.client.get_all(AType::Board).await {
            Ok(ok) => {
                debug!("{:?}", ok);
                self.display
                    .print_artifacts(ok, Some(self.format.to_owned()))
            }
            Err(e) => Err(Error::Core(e)),
        }
    }
    async fn details(&mut self, name: Option<String>) -> Result<WekanResult, Error> {
        match &name {
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
                match query.find_board_id(n, &Some(filter)).await {
                    Ok(board_id) => {
                        let board = self.client.get_one::<Details>(&board_id).await.unwrap();
                        self.display
                            .print_details(board, Some(self.format.to_owned()))
                    }
                    Err(_e) => Err(CliError::new_msg("Board name does not exist").as_enum()),
                }
            }
            None => Err(CliError::new_msg("Board name not supplied").as_enum()),
        }
    }
}
impl<'a> Runner<'a> {
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
                            "Successfully created",
                            "Create a list with subcommand 'list create --help'",
                        )
                        .ok()
                    }
                    Err(e) => {
                        debug!("{:?}", e);
                        CliError::new(4, "Failed to create", Constraint::Login(true)).err()
                    }
                }
            }
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
                    match query.find_board_id(n, &self.global_options.filter).await {
                        Ok(board_id) => match client.delete::<ResponseOk>(&board_id).await {
                            Ok(_o) => WekanResult::new_msg("Successfully deleted").ok(),
                            Err(e) => {
                                trace!("{:?}", e);
                                CliError::new_msg("Failed to delete").err()
                            }
                        },
                        Err(_e) => Err(CliError::new_msg("Board name does not exist").as_enum()),
                    }
                }
                None => Err(CliError::new_msg("Board name not supplied").as_enum()),
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
    async fn get_list_by_board_id(
        &mut self,
        o: &WekanResult,
        board_id: &str,
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
            .inquire(AType::List, Some(board_id), None, false)
            .await
        {
            Ok(lists) => {
                trace!("{:?}", lists);
                if !lists.is_empty() {
                    self.display.prepare_output(
                        &(o.get_msg() + "Following lists are available:\n"),
                        lists,
                        None,
                    )
                } else {
                    WekanResult::new_workflow(
                        &(o.get_msg() + "This boards contains no lists"),
                        "Create a list with subcommand 'list create --help'",
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
    use wekan_common::validation::{authentication::Token, user::User};

    #[tokio::test]
    async fn run_no_options_specified() {
        let r_args = RArgs::mock();
        let mut runner = Runner::new(
            Args::mock(Some(String::from("fake-board-title-2")), None, None),
            Client::mock(),
            BConstraint {
                user: Ok(User {
                    name: *Token::mock().id,
                    token: Some(*Token::mock().token),
                }),
            },
            String::new(),
            CliDisplay::new(Vec::new()),
            &r_args,
        );
        let res = <Runner as RootCommandRunner>::run(&mut runner)
            .await
            .unwrap();
        #[cfg(not(feature = "store"))]
        let expected = concat!(
            "ID                 TITLE              MODIFIED_AT        CREATED_AT\n",
            "my-f               fake-board-title   2020-10-12         2020-10-12\n----\n",
            "ID    TITLE\nfake-board-id-1fake-board-title-1\n",
            "fake-board-id-2fake-board-title-2\n\n----\n"
        );
        #[cfg(feature = "store")]
        let expected = concat!(
            "ID                 TITLE              MODIFIED_AT        CREATED_AT\n",
            "my-f               fake-board-title   2020-10-12         2020-10-12\n----\n",
            "ID    TITLE\nstore-fake-board-id-1store-fake-board-title-1\n",
            "store-fake-board-id-2store-fake-board-title-2\n\n----\n"
        );
        assert_eq!(res.get_msg(), expected);
    }

    #[tokio::test]
    async fn run_no_options_details() {
        let r_args = RArgs::mock();
        let mut runner = Runner::new(
            Args::mock(
                Some(String::from("fake-board-title-2")),
                Some(Command::Details(SDetails {})),
                None,
            ),
            Client::mock(),
            BConstraint {
                user: Ok(User {
                    name: *Token::mock().id,
                    token: Some(*Token::mock().token),
                }),
            },
            String::new(),
            CliDisplay::new(Vec::new()),
            &r_args,
        );
        let res = <Runner as RootCommandRunner>::run(&mut runner)
            .await
            .unwrap();
        let expected = concat!(
            "ID                 TITLE              MODIFIED_AT        CREATED_AT\n",
            "my-f               fake-board-title   2020-10-12         2020-10-12\n----\n"
        );
        assert_eq!(res.get_msg(), expected);
    }

    #[tokio::test]
    async fn run_no_options_create() {
        let r_args = RArgs::mock();
        let mut runner = Runner::new(
            Args::mock(
                Some(String::from("fake-title2")),
                Some(Command::Create(Create {
                    title: String::from("new-board"),
                })),
                None,
            ),
            Client::mock(),
            BConstraint {
                user: Ok(User {
                    name: *Token::mock().id,
                    token: Some(*Token::mock().token),
                }),
            },
            String::new(),
            CliDisplay::new(Vec::new()),
            &r_args,
        );
        let res = <Runner as RootCommandRunner>::run(&mut runner)
            .await
            .unwrap();
        assert_eq!(res.get_msg(), "Successfully created");
    }

    #[tokio::test]
    async fn run_no_options_remove() {
        let r_args = RArgs::mock();
        let mut runner = Runner::new(
            Args::mock(
                Some(String::from("fake-board-title-1")),
                Some(Command::Remove(Remove {})),
                None,
            ),
            Client::mock(),
            BConstraint {
                user: Ok(User {
                    name: *Token::mock().id,
                    token: Some(*Token::mock().token),
                }),
            },
            String::new(),
            CliDisplay::new(Vec::new()),
            &r_args,
        );
        let res = <Runner as RootCommandRunner>::run(&mut runner)
            .await
            .unwrap();
        assert_eq!(res.get_msg(), "Successfully deleted");
    }

    #[tokio::test]
    async fn run_with_special_output() {
        #[cfg(feature = "store")]
        let r_args = RArgs::mock_with(false, false, "long", "b:5");
        #[cfg(not(feature = "store"))]
        let r_args = RArgs::mock_with(false, "long", "b:5");

        let mut runner = Runner::new(
            Args::mock(Some(String::from("fake-board-title-2")), None, None),
            Client::mock(),
            BConstraint {
                user: Ok(User {
                    name: *Token::mock().id,
                    token: Some(*Token::mock().token),
                }),
            },
            String::from("long"),
            CliDisplay::new(Vec::new()),
            &r_args,
        );
        let res = <Runner as RootCommandRunner>::run(&mut runner)
            .await
            .unwrap();

        #[cfg(not(feature = "store"))]
        let expected = concat!(
            "ID                 TITLE              MODIFIED_AT        CREATED_AT\n",
            "my-fake-board-id   fake-board-title   2020-10-12         2020-10-12\n----\n",
            "ID    TITLE\nfake-board-id-1fake-board-title-1\n",
            "fake-board-id-2fake-board-title-2\n\n----\n"
        );
        #[cfg(feature = "store")]
        let expected = concat!(
            "ID                 TITLE              MODIFIED_AT        CREATED_AT\n",
            "my-fake-board-id   fake-board-title   2020-10-12         2020-10-12\n----\n",
            "ID    TITLE\nstore-fake-board-id-1store-fake-board-title-1\n",
            "store-fake-board-id-2store-fake-board-title-2\n\n----\n"
        );
        assert_eq!(res.get_msg(), expected);
    }
}
