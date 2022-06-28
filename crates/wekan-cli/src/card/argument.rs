use crate::{
    command::{
        ArgumentRequester, ArtifactName, CommonCommandRequester, CreateSubcommand,
        SubCommandValidator,
    },
    error::{CliError, Error, Transform},
    subcommand::{Archive, CommonCommand, Details, Inspect, List, Remove},
};
use chrono::prelude::*;
use clap::{Args as ClapArgs, Subcommand};
use wekan_cli_derive::WekanArgs;
use wekan_common::http::common::Create;

#[derive(ClapArgs, Debug, Clone, WekanArgs)]
#[clap(version = "0.1.0", about = "Manage tasks")]
pub struct Args {
    #[clap(short, long, help = "Card name")]
    pub name: Option<String>,
    #[clap(short = 'b', long, help = "Board name")]
    pub board: String,
    #[clap(short = 'l', long, help = "List name")]
    pub list: String,
    #[clap(short, long, parse(from_flag), help = "Show the details of the object")]
    raw: bool,
    /// Move the item to the next column. Optional: Specify a status to move the item to.
    #[clap(subcommand)]
    pub command: Option<Command>,
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
/// The following commands are available:
#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    Ls(List),
    Create(CardCreateArgs),
    #[clap(name = "rm")]
    Remove(Remove),
    #[clap(name = "update")]
    Update(UpdateArgs),
    #[clap(name = "mv")]
    Move(CardMoveArgs),
    Archive(Archive),
    Inspect(Inspect),
    Details(Details),
}

#[derive(ClapArgs, Debug, Clone)]
#[clap(version = "0.1.0", about = "Create card")]
pub struct CardCreateArgs {
    /// Card name
    title: String,
    #[clap(short = 'd', long)]
    description: String,
    #[clap(short, long)]
    swimlane_name: Option<String>,
}
impl CreateSubcommand for CardCreateArgs {}
impl Create for CardCreateArgs {
    fn get_title(&self) -> String {
        self.title.to_owned()
    }
    fn get_description(&self) -> String {
        self.description.to_owned()
    }
}

#[derive(ClapArgs, Debug, Clone)]
#[clap(version = "0.1.0", about = "Move card to the next board")]
pub struct CardMoveArgs {
    pub list: String,
}

#[derive(ClapArgs, Debug, Clone)]
#[clap(version = "0.1.0", about = "Update card")]
pub struct UpdateArgs {
    #[clap(short, long, help = "Card sort order")]
    pub sort: Option<f32>,
    #[clap(short = 't', long, help = "Card title")]
    pub title: Option<String>,
    #[clap(short = 'd', long, help = "Card description")]
    pub description: Option<String>,
    #[clap(short = 'l', long, help = "Supply labels to your card")]
    pub labels: Option<String>,
    #[clap(short = 'f', long, validator = valid_time, help = "Format: Gregorian Day in format (YYYY-MM-DD)")]
    pub due_at: Option<String>,
    #[clap(short, long, validator = valid_time, help = "Format: Gregorian in format (YYYY-MM-DD)")]
    pub end_at: Option<String>,
}
fn valid_time(s: &str) -> Result<Date<Utc>, String> {
    if s.len() > 10 {
        Err(String::from("Day format is too long"))
    } else {
        match NaiveDate::parse_from_str(s, "%Y-%m-%e") {
            Ok(d) => Ok(Date::from_utc(d, Utc)),
            Err(_e) => Err(String::from("Not a correct date format YYYY-MM-DD")),
        }
    }
}
