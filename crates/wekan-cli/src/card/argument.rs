use crate::subcommand::{Details, Inspect};
use chrono::prelude::*;
use clap::{Args as ClapArgs, Subcommand};
use std::path::PathBuf;

#[derive(ClapArgs, Debug, Clone)]
#[clap(version = "0.1.0", about = "Manage tasks")]
pub struct Args {
    /// Selected card
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

/// The following commands are available:
#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    Create(CardCreateArgs),
    #[clap(name = "rm")]
    Remove(RemoveArgs),
    #[clap(name = "update")]
    Update(UpdateArgs),
    #[clap(name = "mv")]
    Move(CardMoveArgs),
    Inspect(Inspect),
    Details(Details),
}

#[derive(ClapArgs, Debug, Clone)]
#[clap(version = "0.1.0", about = "Create card")]
pub struct CardCreateArgs {
    /// Selected card
    pub name: String,
    #[clap(short = 'd', long)]
    pub description: String,
    #[clap(short, long)]
    swimlane_name: Option<String>,
}

#[derive(ClapArgs, Debug, Clone)]
#[clap(version = "0.1.0", about = "Move card to the next status")]
pub struct CardMoveArgs {
    /// Selected card
    pub name: String,
    pub list: String,
}

#[derive(ClapArgs, Debug, Clone)]
#[clap(version = "0.1.0", about = "Remove card from the board")]
pub struct RemoveArgs {
    /// Selected card
    pub name: String,
}

#[derive(ClapArgs, Debug, Clone)]
#[clap(version = "0.1.0", about = "Update card from the board")]
pub struct UpdateArgs {
    /// Selected card
    pub current_name: String,
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
    #[clap(
        short = 'c',
        long,
        parse(from_os_str),
        value_name = "FILE",
        help = "Read a YAML file to update the card"
    )]
    pub card_file: Option<PathBuf>,
}
impl RemoveArgs {
    pub fn get_name(&self) -> String {
        self.name.to_owned()
    }
}
impl CardMoveArgs {
    pub fn get_name(&self) -> String {
        self.name.to_owned()
    }
}
impl UpdateArgs {
    pub fn get_name(&self) -> String {
        self.current_name.to_owned()
    }
}
impl Args {
    pub fn get_name(&self) -> String {
        self.name.as_ref().unwrap().to_owned()
    }
}
impl CardCreateArgs {
    pub fn get_name(&self) -> String {
        self.name.to_owned()
    }
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
