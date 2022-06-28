#[cfg(test)]
use crate::tests::mocks::{Artifacts, Mock, Operation};
use crate::{
    board::Args as BArg,
    card::argument::Args as CArg,
    checklist::Args as ChArg,
    config::argument::Args as Config,
    display::CliDisplay,
    error::{CliError, Error, Transform},
    list::Args as LArg,
    resolver::Query,
    result::WekanResult,
    subcommand::{Apply, CommonCommand as Command, Describe, Get, Inspect, Table},
};
use async_trait::async_trait;
use clap::{Args as CArgs, Parser, Subcommand as CSubcommand};
#[cfg(not(test))]
use clap_verbosity_flag::{ErrorLevel, Verbosity};
use log::{debug, info, trace};
use wekan_core::config::ConfigRequester;
#[cfg(not(test))]
use wekan_core::http::operation::{Artifacts, Operation};

use wekan_common::{
    artifact::common::{AType, DeserializeExt, MostDetails},
    http::artifact::{DetailsResponse, ResponseOk},
};
/// Wekan CLI
#[derive(Parser, Debug)]
#[clap(
    author = "ConfusedOnCOFFEE<me@confusedoncoffee.com>",
    version,
    about = "CLI to manage Wekan users, boards, lists, cards...",
    long_about = "Log in, create contexts and create, update and delete artifacts"
)]
pub struct WekanParser {
    #[clap(flatten)]
    pub delegate: Args,
    #[clap(subcommand)]
    pub command: Subcommand,
}

/// The following commands are avsailable:
#[derive(CSubcommand, Debug, Clone)]
pub enum Subcommand {
    Config(Config),
    Board(BArg),
    Card(CArg),
    List(LArg),
    Checklist(ChArg),
    Table(Table),
    Get(Get),
    Describe(Describe),
    Inspect(Inspect),
    Apply(Apply),
}

#[derive(CArgs, Debug)]
#[clap(name = "wekan-cli", version = "0.1.0", about = "Common artifact args")]
pub struct Args {
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
    pub output_format: Option<String>,
    #[clap(
        short = 'f',
        long,
        help = "Filter out artifacts by id",
        long_help = "Filter out artifacts by id in format: b:..,l:..,c:.. This overrules name argument"
    )]
    pub filter: Option<String>,
    #[clap(flatten)]
    #[cfg(not(test))]
    pub verbose: Verbosity<ErrorLevel>,
}

#[cfg(test)]
impl Mock for Args {
    fn mock() -> Self {
        Self {
            no_recommendations: true,
            #[cfg(feature = "store")]
            no_store: false,
            output_format: None,
            filter: None,
        }
    }
}
#[cfg(test)]
impl Args {
    #[cfg(not(feature = "store"))]
    pub fn mock_with(r: bool, o: &str, f: &str) -> Self {
        Self {
            no_recommendations: r,
            output_format: Some(o.to_string()),
            filter: Some(f.to_string()),
        }
    }
    #[cfg(feature = "store")]
    pub fn mock_with(r: bool, s: bool, o: &str, f: &str) -> Self {
        Self {
            no_recommendations: r,
            no_store: s,
            output_format: Some(o.to_string()),
            filter: Some(f.to_string()),
        }
    }
}
#[async_trait]
pub trait ArtifactCommand<'a, A, H, C> {
    fn new(
        args: A,
        client: H,
        constraint: C,
        format: String,
        display: CliDisplay,
        global_options: &'a Args,
    ) -> Self;
}

#[async_trait]
pub trait BaseCommand<A, H> {
    fn new(args: A, client: H) -> Self;
}

pub trait SubCommandValidator<C> {
    fn get_command(&self) -> Option<C>;
}
pub trait CommonCommandRequester<C>: std::marker::Send + SubCommandValidator<C> {
    fn get_common_command(&self) -> Option<Command>;
    fn has_subcommand(&self) -> bool {
        match self.get_command() {
            Some(_c) => true,
            None => false,
        }
    }
}
#[async_trait]
pub trait ArgumentRequester<C> {
    type Args: ArtifactName + CommonCommandRequester<C> + std::marker::Send + std::marker::Sync;
    fn get_argument(&self) -> Self::Args;
}
#[async_trait]
pub trait ArtifactName {
    fn get_name(&self) -> Result<String, Error>;
}

pub trait CreateSubcommand:
    CArgs + wekan_common::http::common::Create + std::marker::Sync + std::marker::Send
{
}
#[async_trait]
pub trait RootCommandRunner<'a, R: DetailsResponse, C: std::marker::Send>:
    Operator<'a> + ArgumentRequester<C>
{
    async fn run(&mut self) -> Result<WekanResult, Error> {
        let args = self.get_argument();
        match <<Self as ArgumentRequester<C>>::Args as SubCommandValidator<C>>::get_command(&args) {
            Some(_cmd) => self.use_specific_command().await,
            None => self.default().await,
        }
    }
    async fn default(&mut self) -> Result<WekanResult, Error> {
        match self.get_argument().get_name() {
            Ok(n) => self.details::<R>(Some(n.to_string())).await,
            Err(_e) => self.use_ls().await,
        }
    }
    async fn use_common_command(&mut self) -> Result<WekanResult, Error> {
        info!("use_commoncommand");
        let args = self.get_argument();
        match args.get_common_command() {
            Some(c) => match c {
                Command::Ls(_ls) => self.use_ls().await,
                Command::Create(c) => self.use_create(&c).await,
                Command::Remove(_r) => match args.get_name() {
                    Ok(n) => self.remove(Some(n)).await,
                    Err(e) => Err(e),
                },
                Command::Inspect(i) => self.use_inspect(&i).await,
                Command::Details(_d) => match args.get_name() {
                    Ok(n) => self.details::<R>(Some(n)).await,
                    Err(e) => Err(e),
                },
            },
            None => self.default().await,
        }
    }
    async fn use_specific_command(&mut self) -> Result<WekanResult, Error>;
    async fn use_ls(&mut self) -> Result<WekanResult, Error>;
    async fn use_create(
        &mut self,
        create_args: &impl CreateSubcommand,
    ) -> Result<WekanResult, Error>;
    async fn use_inspect(&mut self, inspect_args: &Inspect) -> Result<WekanResult, Error>;
    async fn use_archive<
        B: wekan_common::http::artifact::RequestBody,
        MD: DetailsResponse + MostDetails,
    >(
        &mut self,
        body: &B,
    ) -> Result<WekanResult, Error> {
        info!("use_archive");
        match self.get_client().put::<B, ResponseOk>(body).await {
            Ok(_o) => {
                let details = self
                    .get_client()
                    .get_one::<MD>(&body.get_id())
                    .await
                    .unwrap();
                self.get_display().format_most_details(details)
            }
            Err(_e) => CliError::new_msg("Failed to update").err(),
        }
    }
}

#[async_trait]
pub trait Operator<'a>: Fulfillment<'a> + std::marker::Send + std::marker::Sync {
    fn get_type(&self) -> AType;
    fn get_children_type(&self) -> AType;
    async fn find_details_id(&mut self, name: &str) -> Result<String, Error>;

    async fn unwrap_and_find_id(&mut self, name: Option<String>) -> Result<String, Error> {
        info!("unwrap_and_find_id");
        let n = self.unwrap_name(name)?;
        self.find_details_id(&n).await
    }
    fn unwrap_name(&self, name: Option<String>) -> Result<String, Error> {
        info!("unwrap_name");
        match name {
            Some(n) => Ok(n),
            None => Err(CliError::new_msg("Name not supplied '-n'").as_enum()),
        }
    }
    async fn details<R: DetailsResponse>(
        &mut self,
        name: Option<String>,
    ) -> Result<WekanResult, Error> {
        info!("details");
        let id = self.unwrap_and_find_id(name).await?;
        let base_result = self.get_one::<R>(&id).await?;
        self.get_children(&base_result, &id).await
    }

    async fn create<B: wekan_common::http::artifact::RequestBody, R: DeserializeExt>(
        &mut self,
        body: &B,
    ) -> Result<WekanResult, Error> {
        info!("create");
        match self.get_client().create::<B, R>(body).await {
            Ok(ok) => {
                trace!("{:?}", ok);
                WekanResult::new_msg("Successfully created").ok()
            }
            Err(e) => {
                debug!("{:?}", e);
                CliError::new_msg("Failed to create").err()
            }
        }
    }

    async fn remove(&mut self, name: Option<String>) -> Result<WekanResult, Error> {
        info!("remove");
        let id = self.unwrap_and_find_id(name).await?;
        match self.get_client().delete::<ResponseOk>(&id).await {
            Ok(_o) => WekanResult::new_msg("Successfully deleted").ok(),
            Err(e) => {
                trace!("{:?}", e);
                CliError::new_msg("Failed to delete").err()
            }
        }
    }
    async fn get_all(&mut self) -> Result<WekanResult, Error> {
        info!("get_all");
        match self.get_client().get_all(self.get_type()).await {
            Ok(ok) => self
                .get_display()
                .format_vec(ok, Some(self.get_format().to_owned())),
            Err(e) => {
                trace!("{:?}", e);
                Err(Error::Core(e))
            }
        }
    }
    async fn get_one<R: DetailsResponse>(&mut self, id: &str) -> Result<WekanResult, Error> {
        info!("get_one");
        match self.get_client().get_one::<R>(id).await {
            Ok(d) => self
                .get_display()
                .format_base_details(d, &Some(self.get_format().to_owned())),
            Err(e) => {
                trace!("{:?}", e);
                CliError::new_msg("Failed to request details").err()
            }
        }
    }
    async fn get_children(&mut self, o: &WekanResult, id: &str) -> Result<WekanResult, Error> {
        info!("get_children");
        let mut filter = String::new();
        match &self.get_global_options().filter {
            Some(f) => filter.push_str(f),
            None => {}
        };
        #[cfg(feature = "store")]
        let query = Query {
            filter: &filter,
            config: self.get_client().config,
            deny_store_usage: self.get_global_options().no_store,
        };
        #[cfg(not(feature = "store"))]
        let query = Query {
            filter: &filter,
            config: self.get_client().config,
        };
        let childrens = match self.get_children_type() {
            AType::List => {
                query
                    .inquire(self.get_children_type(), Some(id), None, false)
                    .await
            }
            AType::Card => {
                query
                    .inquire(
                        self.get_children_type(),
                        Some(&self.get_client().get_base_id()),
                        Some(id),
                        false,
                    )
                    .await
            }
            AType::Empty => Ok(Vec::new()),
            _ => Ok(Vec::new()),
        };
        match childrens {
            Ok(children) => {
                trace!("{:?}", children);
                if !children.is_empty() {
                    self.get_display().prepare_output(
                        &(o.get_msg() + "Following children are available:\n"),
                        children,
                        None,
                    )
                } else {
                    WekanResult::new_workflow(
                        &(o.get_msg() + "This artifact contains no children"),
                        "Create a children using the subcommand",
                    )
                    .ok()
                }
            }
            Err(e) => Err(e),
        }
    }
}

pub trait Fulfillment<'a> {
    fn get_client(&mut self) -> wekan_core::client::Client;
    fn get_global_options(&mut self) -> &'a Args;
    fn get_display(&mut self) -> CliDisplay;
    fn get_format(&mut self) -> &str;
}
