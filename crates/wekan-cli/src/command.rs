use crate::{
    error::kind::Error,
    result::kind::WekanResult,
    subcommand::{Describe, Get, Inspect, Table},
};
use async_trait::async_trait;
use clap::{Args, Parser, Subcommand};
use clap_verbosity_flag::{ErrorLevel, Verbosity};
use wekan_common::artifact::common::Artifact;

use crate::{
    board::argument::Args as BArg, card::argument::Args as CArg, config::argument::Args as Config,
    display::CliDisplay, list::argument::Args as LArg,
};

/// Wekan CLI
#[derive(Parser, Debug)]
#[clap(author, version, about = "CLI to manage Wekan users, boards, lists, cards...", long_about = None)]
pub struct WekanParser {
    #[clap(flatten)]
    pub delegate: Root,
    #[clap(subcommand)]
    pub command: RootCmds,
}

/// The following commands are available:
#[derive(Subcommand, Debug, Clone)]
pub enum RootCmds {
    Config(Config),
    Board(BArg),
    Card(CArg),
    List(LArg),
    Table(Table),
    Get(Get),
    Describe(Describe),
    Inspect(Inspect),
    Ps(LArg),
}

#[derive(Args, Debug)]
#[clap(name = "wekan-cli", version = "0.1.0", about = "Common artifact args.")]
pub struct Root {
    #[clap(
        short = 'r',
        long,
        parse(from_flag),
        help = "Disable next recommended workflow"
    )]
    pub no_recommendations: bool,
    #[clap(
        short = 'd',
        long,
        parse(from_flag),
        help = "Disable store for your wekan artifacts"
    )]
    #[cfg(feature = "store")]
    pub no_store: bool,
    #[clap(short = 'o', long, help = "Output format: rust, elisp, long, extended")]
    pub format: Option<String>,
    #[clap(
        short = 'f',
        long,
        help = "Filter out available artifacts by id in format: b:..,l:..,c:.. Be aware that this has a higher order then the name"
    )]
    pub filter: Option<String>,
    #[clap(flatten)]
    pub verbose: Verbosity<ErrorLevel>,
}

#[async_trait]
pub trait ArtifactCommand<A, H, C> {
    fn new(args: A, client: H, constraint: C, format: String, display: CliDisplay) -> Self;
}

#[async_trait]
pub trait BaseCommand<A, H> {
    fn new(args: A, client: H) -> Self;
}

#[async_trait]
pub trait ArtifactName {
    fn get_supplied_name(&self) -> Result<String, Error>;
}
#[async_trait]
pub trait SubCommand {
    async fn get_subcommand(&mut self) -> Result<WekanResult, Error> {
        WekanResult::new_msg("Not implemented.").ok()
    }
    async fn use_subcommand(&mut self) -> Result<WekanResult, Error>;
}

#[async_trait]
pub trait RootCommand {
    async fn run(&mut self) -> Result<WekanResult, Error>;
    async fn use_rootcommand(&mut self, _name: &str) -> Result<WekanResult, Error> {
        WekanResult::new_msg("Not implemented.").ok()
    }
}

#[async_trait]
pub trait CommonRunsSimplified {
    async fn details(&mut self, name: Option<String>) -> Result<WekanResult, Error>;
    async fn list(&mut self) -> Result<WekanResult, Error>;
}
#[async_trait]
pub trait CommonRuns {
    async fn details(&mut self, a: &Artifact) -> Result<WekanResult, Error>;
    async fn list<'a>(&mut self, a: &'a [Artifact]) -> Result<WekanResult, Error>;
}
