extern crate log;
#[cfg(feature = "store")]
use async_trait::async_trait;
use clap::Parser;
use log::{debug, error, info, trace, Level};
use wekan_core::log::Logger;
use wekan_core::{
    client::{BoardApi, CardApi, Client, ListApi, LoginClient},
    http::{operation::Artifacts, preflight_request::Client as PFRClient},
    config::{MandatoryConfig, UserConfig},
};

use crate::{
    resolver::Query,
    subcommand::{Table as TArgs, Describe, Inspect},
    board::{argument::Args as BArgs, runner::Runner as BRunner},
    card::{argument::Args as CArgs, runner::Runner as CRunner},
    command::{WekanParser, ArtifactCommand, BaseCommand, RootCommand, RootCmds as Command},
    error::kind::{CliError, Error, InputError, Transform},
    list::{argument::Args as LArgs, runner::Runner as LRunner},
    result::kind::WekanResult,
    config::runner::Runner as ConfigRunner,
    display::CliDisplay,
};
use wekan_common::{
    artifact::common::{AType, Artifact, Base},
    artifact::{board::Details as BDetails, card::Details as CDetails, list::Details as LDetails},
    validation::{
        constraint::{
            BoardConstraint as BConstraint, CardConstraint as CConstraint,
            ListConstraint as LConstraint,
        },
        user::User,
    },
};

#[cfg(feature = "store")]
use wekan_core::persistence::{
    store::Butler
};
#[cfg(feature = "store")]
use crate::config::context::ReadContext;


pub struct Runner {
    pub parser: WekanParser,
    pub client: LoginClient,
    // pub cache: Option<Cache>,
    pub format: String,
}

impl<'a> Runner {
    pub async fn new() -> Self {
        let parser = WekanParser::parse();
        match parser.delegate.verbose.log_level() {
            Some(Level::Info) => Logger::init(false).unwrap(),
            Some(_) => Logger::init(true).unwrap(),
            None => Logger::init(true).unwrap(),
        };
        let user_config: UserConfig = UserConfig::new();
        #[cfg(feature = "store")]
        let user_config: UserConfig = match user_config.read_context().await {
            Ok(c) => c,
            Err(e) => {
                debug!("{:?}", e);
                UserConfig::new()
            }
        };
        debug!("{:?}", user_config);
        let format = match parser.delegate.format {
            Some(ref f) => f.to_owned(),
            None => "terminal".to_string(),
        };
        debug!("Config done");
        trace!("{:?}", user_config);
        Runner {
            parser,
            client: LoginClient::new(user_config),
            // cache: None,
            format,
        }
    }

    pub async fn run(&mut self) -> Result<WekanResult, Error> {
        debug!("run");
        match &self.parser.command {
            Command::Config(c) => {
                let mut config = ConfigRunner::new(c.clone(), self.client.clone());
                config.run().await
            }
            l => {
                self.client.healthcheck().await?;
                debug!("Artifact command");
                debug!("Initial login done.");
                match &l {
                    Command::Board(b) => self.run_board(b).await,
                    Command::List(l) => self.run_list(l).await,
                    Command::Card(c) => self.run_card(c).await,
                    Command::Table(t) => self.run_table(t).await,
                    Command::Inspect(i) => self.run_inspect(i).await,
                    Command::Describe(d) => self.run_describe(d).await,
                    _ => WekanResult::new_msg("Not implemented.").ok(),
                }
            }
        }
    }

    async fn run_board(&'a self, board_args: &BArgs) -> Result<WekanResult, Error> {
        let client = <Client as BoardApi>::new(self.client.config.clone());
        let constraint =  BConstraint {
            user: Ok(User {
                name: *self.client.config.usertoken.as_ref().unwrap().id.to_owned(),
                token: Some(
                    *self
                        .client
                        .config
                        .usertoken
                        .as_ref()
                        .unwrap()
                        .token
                        .to_owned(),
                ),
            }),
        };
        let mut runner: BRunner = BRunner::new(
            board_args.clone(),
            client,
            constraint,
            self.format.to_owned(),
        );
        runner.run().await
    }
    async fn run_list(&self, l_args: &LArgs) -> Result<WekanResult, Error> {
        info!("run_list_command");
        if !l_args.board.is_empty() {
            #[cfg(feature = "store")]
            let parser = WekanParser::parse();
            #[cfg(feature = "store")]
            let mut query = Query {
                config: self.client.config.clone(),
                deny_store_usage: parser.delegate.no_store,
            };
            #[cfg(not(feature = "store"))]
            let mut query = Query {
                config: self.client.config.clone(),
            };
            let format = &self.parser.delegate.filter;
            match query.find_board_id(&l_args.board.to_string(), format).await {
                Ok(id) => {
                    let constraint = LConstraint {
                        board: Artifact {
                            _id: id,
                            title: l_args.board.to_string(),
                            r#type: AType::Board,
                        },
                    };
                    debug!("Constraint for command list: {:?}", constraint);
                    let client =
                        <Client as ListApi>::new(self.client.config.clone(), &constraint.board._id);
                    trace!("{:?}", client);
                    let mut runner: LRunner =
                        LRunner::new(l_args.clone(), client, constraint, self.format.to_owned());
                    runner.apply().await
                }
                Err(e) => Err(e),
            }
        } else {
            WekanResult::new_exit(
                "Board name not found in cache or given as argument.",
                2,
                None,
            )
            .ok()
        }
    }

    async fn run_card(&self, c_args: &CArgs) -> Result<WekanResult, Error> {
        let constraint = CConstraint {
            board: Some(Artifact {
                _id: String::new(),
                title: c_args.board.to_owned(),
                r#type: AType::Board,
            }),
            list: Some(Artifact {
                _id: String::new(),
                title: c_args.list.to_owned(),
                r#type: AType::List,
            }),
        };

        let parser = WekanParser::parse();
        #[cfg(feature = "store")]
        let query = Query {
            config: self.client.config.clone(),
            deny_store_usage: parser.delegate.no_store,
        };
        #[cfg(not(feature = "store"))]
        let query = Query {
            config: self.client.config.clone(),
        };
        let filter = parser.delegate.filter;
        let client = <Client as CardApi>::new(
            self.client.config.clone(),
            &constraint.board.as_ref().unwrap()._id,
            &constraint.list.as_ref().unwrap()._id,
        );
        let mut runner: CRunner = CRunner::new(
            c_args.clone(),
            client.clone(),
            constraint,
            self.format.to_owned(),
            query,
            filter
        );
        runner.run().await
    }

    async fn run_table(&self, table_args: &TArgs) -> Result<WekanResult, Error> {
        #[cfg(feature = "store")]
        let parser = WekanParser::parse();
        #[cfg(feature = "store")]
        let mut query = Query {
            config: self.client.config.clone(),
            deny_store_usage: parser.delegate.no_store,
        };
        #[cfg(not(feature = "store"))]
        let mut query = Query {
            config: self.client.config.clone(),
        };
        match query
            .find_board_id(&table_args.name, &table_args.filter)
            .await
        {
            Ok(board_id) => {
                match query
                    .request_artifacts(AType::List, &board_id, &String::new())
                    .await
                {
                    Ok(lists) => {
                        trace!("{:?}", lists);
                        let mut iterator = lists.iter();
                        let mut cards_of_lists = Vec::new();
                        if !lists.is_empty() {
                            for r in iterator.by_ref() {
                                trace!("List: {:?}", r.get_id());
                                match query
                                    .request_artifacts(AType::Card, &board_id, &r.get_id())
                                    .await
                                {
                                    Ok(cards) => {
                                        trace!("{:?}", cards);
                                        cards_of_lists.push(cards)
                                    }
                                    Err(_e) => {
                                        cards_of_lists.push(Vec::new())
                                    }
                                };
                            }
                            <Runner as CliDisplay>::print_table(lists, cards_of_lists)
                        } else {
                            <Runner as CliDisplay>::print_table(lists, Vec::new())
                        }
                    }
                    Err(_e) => Err(CliError::new_msg("Lists don't exist.").as_enum()),
                }
            }
            Err(_e) => Err(CliError::new_msg("Name does not exist.").as_enum()),
        }
    }

    async fn run_inspect(&self, i: &Inspect) -> Result<WekanResult, Error> {
        let mut v: Vec<&str> = i.id.split_terminator('/').collect();
        debug!("describe");
        trace!("Vector: {:?}", v);
        if v.len() != 2 {
            WekanResult::new_msg("Format not correct type/id.").ok()
        } else {
            // filter bad typoes bJdaNK9KmbJqLgRzE
            let id = v.remove(1);
            self.verify_id_length(id)?;
            match v.remove(0) {
                "board" | "b" => {
                    let mut client =
                        <Client as BoardApi>::new(self.client.config.clone());
                    match client.get_one::<BDetails>(id).await {
                        Ok(b) => <Runner as CliDisplay>::print_details(
                            b,
                            Some(self.format.to_owned()),
                        ),
                        Err(e) => {
                            error!("Error: {:?}", e);
                            WekanResult::new_msg("Artifact not found.").ok()
                        }
                    }
                }
                "list" | "l" => match &i.delegate.board_id {
                    Some(b_id) => {
                        self.verify_id_length(b_id)?;
                        let mut client = <Client as ListApi>::new(
                            self.client.config.clone(),
                            b_id,
                        );
                        let artifact =
                            client.get_one::<LDetails>(v.remove(0)).await.unwrap();
                        <Runner as CliDisplay>::print_details(
                            artifact,
                            Some(self.format.to_owned()),
                        )
                    }
                    None => {
                        WekanResult::new_msg("Board id needs to be supplied.").ok()
                    }
                },
                "card" | "c" => match &i.delegate.board_id {
                    Some(b_id) => {
                        self.verify_id_length(b_id)?;
                        match &i.delegate.list_id {
                            Some(l_id) => {
                                self.verify_id_length(l_id)?;
                                let mut client = <Client as CardApi>::new(
                                    self.client.config.clone(),
                                    b_id,
                                    l_id,
                                );
                                let artifact = client
                                    .get_one::<CDetails>(v.remove(2))
                                    .await
                                    .unwrap();
                                <Runner as CliDisplay>::print_details(
                                    artifact,
                                    Some(self.format.to_owned()),
                                )
                            }
                            None => WekanResult::new_msg(
                                "List id needs to be supplied.",
                            )
                            .ok(),
                        }
                    }
                    None => {
                        WekanResult::new_msg("Board id needs to be supplied.").ok()
                    }
                },
                _ => WekanResult::new_workflow(
                    "Type does not match.",
                    "Fix your type or look for the type/id combination.",
                )
                .ok(),
            }
        }
    }

    async fn run_describe(&self, d: &Describe) -> Result<WekanResult, Error> {
        let mut v: Vec<&str> = d.resource.split_terminator('/').collect();
        debug!("describe");
        trace!("Vector: {:?}", v);
        #[cfg(feature = "store")]
        let mut query = Query {
            config: self.client.config.clone(),
            deny_store_usage: self.parser.delegate.no_store,
        };
        #[cfg(not(feature = "store"))]
        let mut query = Query {
            config: self.client.config.clone(),
        };
        let filter = &self.parser.delegate.filter;
        if v.len() != 2 {
            WekanResult::new_msg("Format not correct type/id.").ok()
        } else {
            // filter bad typoes bJdaNK9KmbJqLgRzE
            let id = v.remove(1);
            self.verify_id_length(id)?;
            match v.remove(0) {
                "board" | "b" => {
                    let mut client =
                        <Client as BoardApi>::new(self.client.config.clone());
                    match query.find_board_id(v.remove(0), &filter).await {
                        Ok(board_id) => {
                            let board = client.get_one::<BDetails>(&board_id).await.unwrap();
                            <Runner as CliDisplay>::print_details(board, None)
                        }
                        Err(_e) => Err(CliError::new_msg("Board name does not exist").as_enum()),
                    }
                }
                "list" | "l" => match &d.delegate.board_id {
                    Some(b_id) => {
                        self.verify_id_length(b_id)?;
                        let mut client = <Client as ListApi>::new(
                            self.client.config.clone(),
                            b_id,
                        );
                        match query.find_list_id(b_id, v.remove(0), &filter).await {
                            Ok(l_id) => {
                                let board = client.get_one::<LDetails>(&l_id).await.unwrap();
                                <Runner as CliDisplay>::print_details(board, None)
                            }
                            Err(_e) => Err(CliError::new_msg("Board name does not exist").as_enum()),
                        }
                    }
                    None => {
                        WekanResult::new_msg("Board id needs to be supplied.").ok()
                    }
                },
                "card" | "c" => match &d.delegate.board_id {
                    Some(b_id) => {
                        self.verify_id_length(b_id)?;
                        match &d.delegate.list_id {
                            Some(l_id) => {
                                self.verify_id_length(l_id)?;
                                let mut client = <Client as CardApi>::new(
                                    self.client.config.clone(),
                                    b_id,
                                    l_id,
                                );
                                match query.find_card_id(b_id, l_id, v.remove(0), &filter).await {
                                    Ok(c_id) => {
                                        let board = client.get_one::<CDetails>(&c_id).await.unwrap();
                                        <Runner as CliDisplay>::print_details(board, None)
                                    }
                                    Err(_e) => Err(CliError::new_msg("Board name does not exist").as_enum()),
                                }
                            }
                            None => WekanResult::new_msg(
                                "List id needs to be supplied.",
                            )
                            .ok(),
                        }
                    }
                    None => {
                        WekanResult::new_msg("Board id needs to be supplied.").ok()
                    }
                },
                _ => WekanResult::new_workflow(
                    "Type does not match.",
                    "Fix your type or look for the type/id combination.",
                )
                .ok(),
            }
        }
    }

    fn verify_id_length(&self, id: &str) -> Result<bool, Error> {
        if id.len() == 17 {
            Ok(true)
        } else {
            Err(InputError::new_msg("Id is not of specified length").as_enum())
        }
    }
}

impl CliDisplay for Runner {}

#[cfg(feature = "store")]
#[async_trait]
impl ReadContext for UserConfig {

    async fn read_context(&self) -> Result<UserConfig, Error> {
        info!("read_context");
        debug!("{:?}", self.get_path().to_owned());
        match tokio::fs::read(self.get_path().to_owned() + "/config").await {
            Ok(v) => match String::from_utf8_lossy(&v).parse::<String>() {
                Ok(s) => {
                    trace!("{:?}", s);
                    match serde_yaml::from_slice::<UserConfig>(&v) {
                        Ok(c) => {
                            trace!("Read succesfully: {:?}", c);
                            Ok(c)
                        }
                        Err(e) => Err(Error::Yaml(e)),
                    }
                }
                Err(_e) => Ok(UserConfig::new()),
            },
            Err(e) => Err(Error::Io(e)),
        }
    }
}
