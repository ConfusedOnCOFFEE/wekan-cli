use crate::command::CreateSubcommand;
use clap::{Args as ClapArgs, Subcommand};
use std::path::PathBuf;
use wekan_common::{artifact::common::AType, http::common::Create as CreateTrait};
#[derive(Subcommand, Debug, Clone)]
#[clap(about = "The following commands are available:")]
pub enum CommonCommand {
    Ls(List),
    Details(Details),
    Create(Create),
    Inspect(Inspect),
    #[clap(name = "rm")]
    Remove(Remove),
}

#[derive(ClapArgs, Debug, Clone)]
#[clap(
    about = "Get an artifact",
    long_about = "Get an artifact with an alternative format"
)]
pub struct Get {
    // Kind of artifact
    kind: AType,
    // Name
    name: String,
    #[clap(flatten)]
    mandatory: Delegate,
}

#[derive(ClapArgs, Debug, Clone)]
#[clap(
    about = "List all artifacts",
    long_about = "Shows all desired artifacts in a table view"
)]
pub struct List {}
#[derive(ClapArgs, Debug, Clone)]
#[clap(
    about = "Show the details",
    long_about = "Show the details in a table view"
)]
pub struct Details {}

#[derive(ClapArgs, Debug, Clone)]
#[clap(
    about = "Remove an artifact",
    long_about = "Remove an artifact. This action can not be reverted"
)]
pub struct Remove {}

#[derive(ClapArgs, Debug, Clone)]
#[clap(
    about = "Create an artifact",
    long_about = "Create an artifact with a simply title"
)]
pub struct Create {
    /// Artifact title
    pub title: String,
}

impl CreateSubcommand for Create {}
impl CreateTrait for Create {
    fn get_title(&self) -> String {
        self.title.to_owned()
    }
    fn get_description(&self) -> String {
        String::from("Description not available")
    }
}
#[derive(ClapArgs, Debug, Clone)]
pub struct Delegate {
    #[clap(short = 'b', long, group = "base_arg", help = "Board id")]
    pub board_id: Option<String>,
    #[clap(short = 'l', long, requires = "base_arg", help = "List id")]
    pub list_id: Option<String>,
    #[clap(short = 'c', long, requires = "base_arg", help = "Card id")]
    pub card_id: Option<String>,
}
#[derive(ClapArgs, Debug, Clone)]
#[clap(
    about = "Describe artifact by id",
    long_about = "Use ID instead of a name. This is more peformant"
)]
pub struct Inspect {
    /// Artifact id
    pub id: String,
    #[clap(flatten)]
    pub delegate: Delegate,
}

#[derive(ClapArgs, Debug, Clone)]
#[clap(
    about = "Describe artfifact",
    long_about = "Use the kubernetes syntax to get details of an artifact"
)]
pub struct Describe {
    /// Artifact type and name (format: type/name)
    pub resource: String,
    #[clap(flatten)]
    pub delegate: Delegate,
}

#[derive(ClapArgs, Debug, Clone)]
#[clap(
    about = "Show a board table",
    long_about = "Show the board with available lists and cards"
)]
pub struct Table {
    /// Board name
    pub name: String,
    #[clap(short = 'f', long, help = "Filter by b:,l:")]
    pub filter: Option<String>,
}

#[derive(ClapArgs, Debug, Clone)]
#[clap(
    about = "Apply a change to an artifact",
    long_about = "Use a file to update an artifact"
)]
pub struct Apply {
    /// Artifact file
    task_file: PathBuf,
}

#[derive(ClapArgs, Debug, Clone)]
#[clap(
    about = "Archive an artifact but currently not available. See --help",
    long_about = "Currently the API doesn't support this action. Maybe in the future :)"
)]
pub struct Archive {
    #[clap(
        short = 'r',
        long,
        parse(from_flag),
        help = "Restore an archived artifact"
    )]
    pub restore: bool,
}
