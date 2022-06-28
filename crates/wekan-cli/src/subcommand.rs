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
#[clap(version = "0.1.0", about = "Get an artifact")]
pub struct Get {
    // Kind of artifact
    kind: AType,
    // Name
    name: String,
    #[clap(flatten)]
    mandatory: Delegate,
}

#[derive(ClapArgs, Debug, Clone)]
#[clap(version = "0.1.0", about = "List all artifacts")]
pub struct List {}
#[derive(ClapArgs, Debug, Clone)]
#[clap(version = "0.1.0", about = "Show the details")]
pub struct Details {}

#[derive(ClapArgs, Debug, Clone)]
#[clap(version = "0.1.0", about = "Remove an artifact")]
pub struct Remove {}
#[derive(ClapArgs, Debug, Clone)]
#[clap(version = "0.1.0", about = "Create an artifact")]
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
#[clap(version = "0.1.0", about = "Describe artifact by id")]
pub struct Inspect {
    /// Artifact id
    pub id: String,
    #[clap(flatten)]
    pub delegate: Delegate,
}

#[derive(ClapArgs, Debug, Clone)]
#[clap(
    version = "0.1.0",
    about = "Describe artfifact by k8 syntax (type/name)"
)]
pub struct Describe {
    /// Artifact type and name (format: type/name)
    pub resource: String,
    #[clap(flatten)]
    pub delegate: Delegate,
}

#[derive(ClapArgs, Debug, Clone)]
#[clap(version = "0.1.0", about = "Show a board table")]
pub struct Table {
    /// Board name
    pub name: String,
    #[clap(short = 'f', long, help = "Filter by b:,l:")]
    pub filter: Option<String>,
}

#[derive(ClapArgs, Debug, Clone)]
#[clap(version = "0.1.0", about = "Apply a change to an artifact")]
pub struct Apply {
    /// Artifact file
    task_file: PathBuf,
}

#[derive(ClapArgs, Debug, Clone)]
#[clap(
    version = "0.1.0",
    about = "Archive an artifact (API doesn't support it :( )"
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
