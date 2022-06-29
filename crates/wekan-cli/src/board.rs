use crate::{
    command::{
        Args as RArgs, ArgumentRequester, ArtifactCommand, ArtifactName, CommonCommandRequester,
        CreateSubcommand, Fulfillment, Operator, RootCommandRunner, SubCommandValidator,
    },
    display::CliDisplay,
    error::{CliError, Error, Transform},
    resolver::Query,
    result::WekanResult,
    subcommand::{CommonCommand as Command, Inspect},
};
use async_trait::async_trait;
use clap::Args as ClapArgs;
use wekan_cli_derive::{CommonSubcommands, FulfilmentRunner, WekanArgs};
use wekan_common::{
    artifact::{board::Details, common::AType},
    http::board::{CreateBoard, CreatedBoard},
    validation::{authentication::TokenHeader, constraint::BoardConstraint as BConstraint},
};
use wekan_core::client::{BoardApi, Client};

/// Board commands
#[derive(ClapArgs, Debug, Clone, WekanArgs, CommonSubcommands)]
#[clap(
    about = "Manage boards",
    long_about = "Create, remove and show details and children"
)]
pub struct Args {
    #[clap(short, long, help = "Board name")]
    pub name: Option<String>,
    /// ls: Option<String>,
    /// Subcommands for board config, show swimlanes, lists.
    #[clap(subcommand)]
    pub command: Option<Command>,
}

#[cfg(test)]
impl Args {
    pub fn mock(name: Option<String>, command: Option<Command>) -> Self {
        Args { name, command }
    }
}

#[derive(FulfilmentRunner)]
pub struct Runner<'a> {
    pub args: Args,
    pub client: Client,
    pub constraint: BConstraint,
    pub format: String,
    pub display: CliDisplay,
    pub global_options: &'a RArgs,
}

#[async_trait]
impl<'a> Operator<'a> for Runner<'a> {
    async fn find_details_id(&mut self, name: &str) -> Result<String, Error> {
        let mut filter = String::new();
        match &self.global_options.filter {
            Some(f) => filter.push_str(f),
            None => {}
        };
        #[cfg(feature = "store")]
        let mut query = Query {
            filter: &filter,
            config: self.get_client().config,
            deny_store_usage: self.get_global_options().no_store,
        };
        #[cfg(not(feature = "store"))]
        let mut query = Query {
            filter: &filter,
            config: self.get_client().config,
        };
        query.find_board_id(name).await
    }

    fn get_type(&self) -> AType {
        AType::Board
    }

    fn get_children_type(&self) -> AType {
        AType::List
    }
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
impl<'a> RootCommandRunner<'a, Details, Command> for Runner<'a> {
    async fn use_specific_command(&mut self) -> Result<WekanResult, Error> {
        self.use_common_command().await
    }
    async fn use_ls(&mut self) -> Result<WekanResult, Error> {
        self.client
            .set_base(&("users/".to_owned() + &self.client.get_user_id() + "/boards"));
        self.get_all().await
    }
    async fn use_create(
        &mut self,
        create_args: &impl CreateSubcommand,
    ) -> Result<WekanResult, Error> {
        let client = self.client.clone();
        let body = CreateBoard {
            _id: String::new(),
            title: create_args.get_title(),
            owner: client.get_user_id(),
            permission: Some(String::from("private")),
            color: None,
            is_admin: None,
            is_active: None,
            is_no_comments: None,
            is_comment_only: None,
            is_worker: None,
        };
        self.create::<CreateBoard, CreatedBoard>(&body).await
    }
    async fn use_inspect(&mut self, inspect_args: &Inspect) -> Result<WekanResult, Error> {
        self.get_one::<Details>(&inspect_args.id).await
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
            Args::mock(Some(String::from("fake-board-title-2")), None),
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
        let res = runner.run().await.unwrap();
        #[cfg(not(feature = "store"))]
        let expected = concat!(
            "ID                 TITLE              MODIFIED_AT        CREATED_AT\n",
            "my-f               fake-board-title   2020-10-12         2020-10-12\n----\n",
            "Following children are available:\n",
            "ID    TITLE\nfake  fake-list-title-1\nfake  fake-list-title-2\n\n----"
        );
        #[cfg(feature = "store")]
        let expected = concat!(
            "ID                 TITLE              MODIFIED_AT        CREATED_AT\n",
            "my-f               fake-board-title   2020-10-12         2020-10-12\n----\n",
            "Following children are available:\n",
            "ID    TITLE\nstor  store-fake-list-title-1\nstor  store-fake-list-title-2\n\n----"
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
        let res = runner.run().await.unwrap();
        #[cfg(not(feature = "store"))]
        let expected = concat!(
            "ID                 TITLE              MODIFIED_AT        CREATED_AT\n",
            "my-f               fake-board-title   2020-10-12         2020-10-12\n----\n",
            "Following children are available:\n",
            "ID    TITLE\nfake  fake-list-title-1\nfake  fake-list-title-2\n\n----"
        );
        #[cfg(feature = "store")]
        let expected = concat!(
            "ID                 TITLE              MODIFIED_AT        CREATED_AT\n",
            "my-f               fake-board-title   2020-10-12         2020-10-12\n----\n",
            "Following children are available:\n",
            "ID    TITLE\nstor  store-fake-list-title-1\nstor  store-fake-list-title-2\n\n----"
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
        let res = runner.run().await.unwrap();
        assert_eq!(res.get_msg(), "Successfully created");
    }

    #[tokio::test]
    async fn run_no_options_remove() {
        let r_args = RArgs::mock();
        let mut runner = Runner::new(
            Args::mock(
                Some(String::from("fake-board-title-1")),
                Some(Command::Remove(Remove {})),
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
        let res = runner.run().await.unwrap();
        assert_eq!(res.get_msg(), "Successfully deleted");
    }

    #[tokio::test]
    async fn run_with_special_output() {
        #[cfg(feature = "store")]
        let r_args = RArgs::mock_with(false, false, "long", "b:5");
        #[cfg(not(feature = "store"))]
        let r_args = RArgs::mock_with(false, "long", "b:5");

        let mut runner = Runner::new(
            Args::mock(Some(String::from("fake-board-title-2")), None),
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
        let res = runner.run().await.unwrap();

        #[cfg(not(feature = "store"))]
        let expected = concat!(
            "ID                 TITLE              MODIFIED_AT        CREATED_AT\n",
            "my-fake-board-id   fake-board-title   2020-10-12         2020-10-12\n----\n",
            "Following children are available:\n",
            "ID    TITLE\nfake  fake-list-title-1\nfake  fake-list-title-2\n\n----"
        );
        #[cfg(feature = "store")]
        let expected = concat!(
            "ID                 TITLE              MODIFIED_AT        CREATED_AT\n",
            "my-fake-board-id   fake-board-title   2020-10-12         2020-10-12\n----\n",
            "Following children are available:\n",
            "ID    TITLE\nstor  store-fake-list-title-1\nstor  store-fake-list-title-2\n\n----"
        );
        assert_eq!(res.get_msg(), expected);
    }
}
