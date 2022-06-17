use clap::{Args as ClapArgs, Subcommand};
use wekan_common::artifact::common::AType;
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
#[clap(version = "0.1.0", about = "List Artifacts")]
pub struct List {
    #[clap(
        short = 't',
        long,
        parse(from_flag),
        help = "Show the artifacts in a table"
    )]
    table: bool,
}

#[derive(ClapArgs, Debug, Clone)]
#[clap(version = "0.1.0", about = "List artifacts")]
pub struct Get {
    // Kind of artifact
    kind: AType,
    // Name
    name: String,
    #[clap(flatten)]
    mandatory: Delegate,
}

#[derive(ClapArgs, Debug, Clone)]
#[clap(version = "0.1.0", about = "Show details")]
pub struct Details {}

#[derive(ClapArgs, Debug, Clone)]
#[clap(version = "0.1.0", about = "Remove artifact")]
pub struct Remove {}
#[derive(ClapArgs, Debug, Clone)]
#[clap(version = "0.1.0", about = "Create artifact")]
pub struct Create {
    /// Board to be created
    pub title: String,
}
#[derive(ClapArgs, Debug, Clone)]
#[clap(
    version = "0.1.0",
    about = "Describe artfifact by k8 syntax (type/name)"
)]
pub struct Delegate {
    #[clap(short = 'b', long, group = "card_arg", help = "Board id")]
    pub board_id: Option<String>,
    #[clap(short = 'l', long, requires = "card_arg", help = "List id")]
    pub list_id: Option<String>,
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
