use crate::{
    command::{
        Args as RArgs, ArgumentRequester, ArtifactCommand, ArtifactName, CommonCommandRequester,
        CreateSubcommand, Fulfillment, Operator, RootCommandRunner, SubCommandValidator,
    },
    display::CliDisplay,
    error::{CliError, Error, Transform},
    resolver::Query,
    result::WekanResult,
    subcommand::{CommonCommand, Details as SDetails, Inspect, List, Remove},
};
use async_trait::async_trait;
use log::info;
use wekan_cli_derive::{FulfilmentRunner, WekanArgs};
use wekan_common::{
    artifact::{checklist::Details, common::AType},
    http::{
        artifact::{CreateArtifact, ResponseOk},
        common::Create,
    },
    validation::constraint::ChecklistConstraint as ChConstraint,
};
use wekan_core::client::{ChecklistApi, Client};

use clap::{Args as ClapArgs, Subcommand};

#[derive(ClapArgs, Debug, Clone, WekanArgs)]
#[clap(
    about = "Manage checklists",
    long_about = "Create, remove and show details"
)]
pub struct Args {
    #[clap(short, long, help = "Checklist name")]
    pub name: Option<String>,
    #[clap(short = 'b', long, help = "Board name")]
    pub board: String,
    #[clap(short = 'l', long, help = "List name")]
    pub list: String,
    #[clap(short = 'c', long, help = "Card name")]
    pub card: String,
    #[clap(subcommand)]
    pub command: Option<Command>,
}

/// The following commands are available:
#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    Ls(List),
    Create(ChecklistCreateArgs),
    #[clap(name = "rm")]
    Remove(Remove),
    Inspect(Inspect),
    Details(SDetails),
}
#[cfg(test)]
impl Args {
    pub fn mock(
        name: Option<String>,
        board: String,
        list: String,
        card: String,
        command: Option<Command>,
    ) -> Self {
        Args {
            name,
            board,
            command,
            list,
            card,
        }
    }
}

impl CommonCommandRequester<Command> for Args {
    fn get_common_command(&self) -> Option<CommonCommand> {
        match &self.command {
            Some(c) => match c {
                Command::Ls(ls) => Some(CommonCommand::Ls(ls.to_owned())),
                Command::Remove(r) => Some(CommonCommand::Remove(r.to_owned())),
                Command::Inspect(i) => Some(CommonCommand::Inspect(i.to_owned())),
                Command::Details(d) => Some(CommonCommand::Details(d.to_owned())),
                _ => None,
            },
            None => None,
        }
    }
}
#[derive(ClapArgs, Debug, Clone)]
#[clap(
    about = "Create checklist",
    long_about = "Create a checklist with title and items"
)]
pub struct ChecklistCreateArgs {
    /// Card name
    title: String,
    #[clap(short = 'i', long)]
    /// Items with komma
    items: String,
}
impl CreateSubcommand for ChecklistCreateArgs {}
impl Create for ChecklistCreateArgs {
    fn get_title(&self) -> String {
        self.title.to_owned()
    }
    fn get_description(&self) -> String {
        String::new()
    }
}

#[derive(FulfilmentRunner)]
pub struct Runner<'a> {
    pub args: Args,
    pub client: Client,
    pub constraint: ChConstraint,
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
        query
            .find_checklist_id(&self.constraint.board._id, &self.constraint.card._id, name)
            .await
    }
    fn get_type(&self) -> AType {
        AType::Checklist
    }

    fn get_children_type(&self) -> AType {
        AType::Empty
    }
}

impl<'a> ArtifactCommand<'a, Args, Client, ChConstraint> for Runner<'a> {
    fn new(
        args: Args,
        client: Client,
        constraint: ChConstraint,
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
        info!("use_specific_command");
        match self.args.command.to_owned() {
            Some(c) => match c {
                Command::Create(c) => self.use_create(&c).await,
                _ => self.use_common_command().await,
            },
            None => CliError::new_msg("Subcommand not implemented").err(),
        }
    }
    async fn use_ls(&mut self) -> Result<WekanResult, Error> {
        self.get_all().await
    }
    async fn use_inspect(&mut self, inspect_args: &Inspect) -> Result<WekanResult, Error> {
        match &inspect_args.delegate.board_id {
            Some(id) => match &inspect_args.delegate.card_id {
                Some(card_id) => {
                    self.client.set_base(id, card_id);
                    self.get_one::<Details>(&inspect_args.id).await
                }
                None => WekanResult::new_msg("Card id needs to be supplied").ok(),
            },
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
        subcommand::{Details as SDetails, Remove},
        tests::mocks::Mock,
    };
    use wekan_common::artifact::common::Artifact;

    #[tokio::test]
    async fn run_no_options_specified() {
        let r_args = RArgs::mock();
        let mut runner = Runner::new(
            Args::mock(
                Some(String::from("fake-checklist-title-2")),
                String::from("fake-board-title-1"),
                String::from("fake-list-title-2"),
                String::from("fake-card-title-2"),
                None,
            ),
            Client::mock(),
            ChConstraint {
                board: Artifact {
                    _id: String::from("fake-board-id-1"),
                    title: String::from("fake-board-title-1"),
                    r#type: AType::Board,
                },
                list: Artifact {
                    _id: String::from("fake-list-id-2"),
                    title: String::from("fake-card-title-2"),
                    r#type: AType::List,
                },
                card: Artifact {
                    _id: String::from("fake-card-id-2"),
                    title: String::from("fake-card-title-2"),
                    r#type: AType::Card,
                },
            },
            String::new(),
            CliDisplay::new(Vec::new()),
            &r_args,
        );
        #[cfg(not(feature = "store"))]
        let expected = concat!(
            "ID                     TITLE                  MODIFIED_AT            CREATED_AT\n",
            "my-f                   fake-checklist-title   2020-10-12             2020-10-12\n----\n",
            "This artifact contains no children"
        );
        #[cfg(feature = "store")]
        let expected = concat!(
            "ID                     TITLE                  MODIFIED_AT            CREATED_AT\n",
            "my-f                   fake-checklist-title   2020-10-12             2020-10-12\n----\n",
            "This artifact contains no children"
        );
        let res = runner.run().await.unwrap();
        assert_eq!(res.get_msg(), expected);
    }

    #[tokio::test]
    async fn run_no_options_details() {
        let r_args = RArgs::mock();
        let mut runner = Runner::new(
            Args::mock(
                Some(String::from("fake-checklist-title-2")),
                String::from("fake-board-title-2"),
                String::from("fake-list-title-2"),
                String::from("fake-card-title-2"),
                Some(Command::Details(SDetails {})),
            ),
            Client::mock(),
            ChConstraint {
                board: Artifact {
                    _id: String::from("fake-board-id-2"),
                    title: String::from("fake-board-title-2"),
                    r#type: AType::Board,
                },
                list: Artifact {
                    _id: String::from("fake-list-id-2"),
                    title: String::from("fake-card-title-2"),
                    r#type: AType::List,
                },
                card: Artifact {
                    _id: String::from("fake-card-id-2"),
                    title: String::from("fake-card-title-2"),
                    r#type: AType::Card,
                },
            },
            String::new(),
            CliDisplay::new(Vec::new()),
            &r_args,
        );
        let res = runner.run().await.unwrap();
        #[cfg(not(feature = "store"))]
        let expected = concat!(
            "ID                     TITLE                  MODIFIED_AT            CREATED_AT\n",
            "my-f                   fake-checklist-title   2020-10-12             2020-10-12\n----\n",
            "This artifact contains no children"
        );
        #[cfg(feature = "store")]
        let expected = concat!(
            "ID                     TITLE                  MODIFIED_AT            CREATED_AT\n",
            "my-f                   fake-checklist-title   2020-10-12             2020-10-12\n----\n",
            "This artifact contains no children"
        );
        assert_eq!(res.get_msg(), expected);
    }

    #[tokio::test]
    async fn run_no_options_create() {
        let r_args = RArgs::mock();
        let mut runner = Runner::new(
            Args::mock(
                Some(String::from("fake-checklist-title-2")),
                String::from("fake-board-title-1"),
                String::from("fake-list-title-2"),
                String::from("fake-card-title-2"),
                Some(Command::Create(ChecklistCreateArgs {
                    title: String::from("new-board"),
                    items: String::from("first,second,third"),
                })),
            ),
            Client::mock(),
            ChConstraint {
                board: Artifact {
                    _id: String::from("fake-board-id-1"),
                    title: String::from("fake-board-title-1"),
                    r#type: AType::Board,
                },
                list: Artifact {
                    _id: String::from("fake-list-id-2"),
                    title: String::from("fake-card-title-2"),
                    r#type: AType::List,
                },
                card: Artifact {
                    _id: String::from("fake-card-id-2"),
                    title: String::from("fake-card-title-2"),
                    r#type: AType::Card,
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
                Some(String::from("fake-checklist-title-2")),
                String::from("fake-board-title-1"),
                String::from("fake-list-title-2"),
                String::from("fake-card-title-2"),
                Some(Command::Remove(Remove {})),
            ),
            Client::mock(),
            ChConstraint {
                board: Artifact {
                    _id: String::from("fake-board-id-1"),
                    title: String::from("fake-board-title-1"),
                    r#type: AType::Board,
                },
                list: Artifact {
                    _id: String::from("fake-list-id-2"),
                    title: String::from("fake-card-title-2"),
                    r#type: AType::List,
                },
                card: Artifact {
                    _id: String::from("fake-card-id-2"),
                    title: String::from("fake-card-title-2"),
                    r#type: AType::Card,
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
                Some(String::from("fake-checklist-title-2")),
                String::from("fake-board-title-2"),
                String::from("fake-list-title-2"),
                String::from("fake-card-title-2"),
                None,
            ),
            Client::mock(),
            ChConstraint {
                board: Artifact {
                    _id: String::from("fake-board-id-2"),
                    title: String::from("fake-board-title-2"),
                    r#type: AType::Board,
                },
                list: Artifact {
                    _id: String::from("fake-list-id-2"),
                    title: String::from("fake-card-title-2"),
                    r#type: AType::List,
                },
                card: Artifact {
                    _id: String::from("fake-card-id-2"),
                    title: String::from("fake-card-title-2"),
                    r#type: AType::Card,
                },
            },
            String::from("long"),
            CliDisplay::new(Vec::new()),
            &r_args,
        );
        #[cfg(not(feature = "store"))]
        let expected = concat!(
            "ID                     TITLE                  MODIFIED_AT            CREATED_AT\n",
            "my-fake-checklist-id   fake-checklist-title   2020-10-12             2020-10-12\n----\n",
            "This artifact contains no children"
        );
        #[cfg(feature = "store")]
        let expected = concat!(
            "ID                     TITLE                  MODIFIED_AT            CREATED_AT\n",
            "my-fake-checklist-id   fake-checklist-title   2020-10-12             2020-10-12\n----\n",
            "This artifact contains no children"
        );
        let res = runner.run().await.unwrap();
        assert_eq!(res.get_msg(), expected);
    }
}
