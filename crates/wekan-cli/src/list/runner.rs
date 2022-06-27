use crate::{
    command::{
        Args as RArgs, ArgumentRequester, ArtifactCommand, CreateSubcommand, Fulfillment, Operator,
        RootCommandRunner,
    },
    display::CliDisplay,
    error::kind::Error,
    list::argument::Args,
    resolver::Query,
    result::kind::WekanResult,
    subcommand::{CommonCommand as Command, Inspect},
};
use async_trait::async_trait;
use wekan_cli_derive::FulfilmentRunner;
use wekan_common::{
    artifact::{common::AType, list::Details},
    http::artifact::{CreateArtifact, ResponseOk},
    validation::constraint::ListConstraint as LConstraint,
};
use wekan_core::client::{Client, ListApi};

#[derive(FulfilmentRunner)]
pub struct Runner<'a> {
    pub args: Args,
    pub client: Client,
    pub constraint: LConstraint,
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
        query.find_list_id(&self.constraint.board._id, name).await
    }
    fn get_type(&self) -> AType {
        AType::List
    }

    fn get_children_type(&self) -> AType {
        AType::Card
    }
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

#[async_trait]
impl<'a> RootCommandRunner<'a, Details, Command> for Runner<'a> {
    async fn use_specific_command(&mut self) -> Result<WekanResult, Error> {
        self.use_common_command().await
    }
    async fn use_ls(&mut self) -> Result<WekanResult, Error> {
        self.get_all().await
    }
    async fn use_inspect(&mut self, inspect_args: &Inspect) -> Result<WekanResult, Error> {
        match &inspect_args.delegate.board_id {
            Some(id) => {
                self.client.set_base(id);
                self.get_one::<Details>(&inspect_args.id).await
            }
            None => WekanResult::new_msg("Board id needs to be supplied").ok(),
        }
    }

    async fn use_create(
        &mut self,
        create_args: &impl CreateSubcommand,
    ) -> Result<WekanResult, Error> {
        let c_a = CreateArtifact {
            _id: String::new(),
            title: create_args.get_title(),
        };
        self.create::<CreateArtifact, ResponseOk>(&c_a).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        subcommand::{Create, Details as SDetails, Remove},
        tests::mocks::Mock,
    };
    use wekan_common::artifact::common::Artifact;

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
        #[cfg(not(feature = "store"))]
        let expected = concat!(
            "ID                TITLE             MODIFIED_AT       CREATED_AT\n",
            "my-f              fake-list-title   2020-10-12        2020-10-12\n----\n",
            "Following children are available:\n",
            "ID    TITLE\nfake  fake-card-title-1\nfake  fake-card-title-2\n\n----"
        );
        #[cfg(feature = "store")]
        let expected = concat!(
            "ID                TITLE             MODIFIED_AT       CREATED_AT\n",
            "my-f              fake-list-title   2020-10-12        2020-10-12\n----\n",
            "Following children are available:\n",
            "ID    TITLE\nstor  store-fake-card-title-1\nstor  store-fake-card-title-2\n\n----"
        );
        let res = runner.run().await.unwrap();
        assert_eq!(res.get_msg(), expected);
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
        let res = runner.run().await.unwrap();
        #[cfg(not(feature = "store"))]
        let expected = concat!(
            "ID                TITLE             MODIFIED_AT       CREATED_AT\n",
            "my-f              fake-list-title   2020-10-12        2020-10-12\n----\n",
            "Following children are available:\n",
            "ID    TITLE\nfake  fake-card-title-1\nfake  fake-card-title-2\n\n----"
        );
        #[cfg(feature = "store")]
        let expected = concat!(
            "ID                TITLE             MODIFIED_AT       CREATED_AT\n",
            "my-f              fake-list-title   2020-10-12        2020-10-12\n----\n",
            "Following children are available:\n",
            "ID    TITLE\nstor  store-fake-card-title-1\nstor  store-fake-card-title-2\n\n----"
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
        let res = runner.run().await.unwrap();
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
        #[cfg(not(feature = "store"))]
        let expected = concat!(
            "ID                TITLE             MODIFIED_AT       CREATED_AT\n",
            "my-fake-list-id   fake-list-title   2020-10-12        2020-10-12\n----\n",
            "Following children are available:\n",
            "ID    TITLE\nfake  fake-card-title-1\nfake  fake-card-title-2\n\n----"
        );
        #[cfg(feature = "store")]
        let expected = concat!(
            "ID                TITLE             MODIFIED_AT       CREATED_AT\n",
            "my-fake-list-id   fake-list-title   2020-10-12        2020-10-12\n----\n",
            "Following children are available:\n",
            "ID    TITLE\nstor  store-fake-card-title-1\nstor  store-fake-card-title-2\n\n----"
        );
        let res = runner.run().await.unwrap();
        assert_eq!(res.get_msg(), expected);
    }
}
