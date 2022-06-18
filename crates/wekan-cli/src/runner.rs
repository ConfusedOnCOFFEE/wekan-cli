extern crate log;
#[cfg(feature = "store")]
use async_trait::async_trait;
use log::{debug, error, info, trace};
use wekan_core::{
    client::{BoardApi, CardApi, Client, ListApi, LoginClient},
    config::{MandatoryConfig, UserConfig},
    http::{preflight_request::Client as PFRClient},
};

use crate::{
    board::{argument::Args as BArgs, runner::Runner as BRunner},
    card::{
        argument::Args as CArgs,
        runner::{NewCardRunner, Runner as CRunner},
    },
    command::{ArtifactCommand, BaseCommand, RootCommandRunner, Subcommand as Command, Args as RArgs},
    config::runner::Runner as ConfigRunner,
    display::CliDisplay,
    error::kind::{CliError, Error, InputError, Transform},
    list::{argument::Args as LArgs, runner::Runner as LRunner},
    resolver::Query,
    result::kind::WekanResult,
    subcommand::{Describe, Inspect, Table as TArgs},
};
use wekan_common::{
    artifact::common::{AType, Artifact, IdReturner},
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
use crate::config::context::ReadContext;
#[cfg(feature = "store")]
use wekan_core::persistence::store::Butler;
#[cfg(not(test))]
use log::Level;
#[cfg(not(test))]
use wekan_core::{
    log::Logger,
    http::operation::Artifacts
};

#[cfg(test)]
use crate::tests::mocks::Artifacts;

pub struct Runner {
    pub client: LoginClient,
    pub format: String,
    pub display: CliDisplay,
    pub subcommands: Command,
    pub global_options: RArgs
}

impl<'a> Runner {
    pub async fn new(r_args: RArgs, subcommands: Command) -> Self {
        #[cfg(not(test))]
        match r_args.verbose.log_level() {
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
        let format = match r_args.output_format {
            Some(ref f) => f.to_owned(),
            None => "terminal".to_string(),
        };
        debug!("Config done");
        trace!("{:?}", user_config);
        let vec: Vec<u8> = Vec::new();
        Runner {
            client: LoginClient::new(user_config),
            format,
            display: CliDisplay::new(vec),
            subcommands,
            global_options: r_args
        }
    }

    pub async fn run(&mut self) -> Result<WekanResult, Error> {
        debug!("run");
        match self.subcommands.to_owned() {
            Command::Config(c) => {
                let mut config = ConfigRunner::new(c.clone(), self.client.clone());
                config.run().await
            }
            l => {
                self.client.healthcheck().await?;
                debug!("Artifact command");
                debug!("Initial login done.");
                match l {
                    Command::Board(b) => self.run_board(&b).await,
                    Command::List(l) => self.run_list(&l).await,
                    Command::Card(c) => self.run_card(&c).await,
                    Command::Table(t) => self.run_table(&t).await,
                    Command::Inspect(i) => self.run_inspect(i).await,
                    Command::Describe(d) => self.run_describe(d).await,
                    _ => WekanResult::new_msg("Not implemented.").ok(),
                }
            }
        }
    }

    async fn run_board(&'a self, board_args: &BArgs) -> Result<WekanResult, Error> {
        let client = <Client as BoardApi>::new(self.client.config.clone());
        let constraint = BConstraint {
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
            self.display.to_owned(),
            &self.global_options
        );
        runner.run().await
    }
    async fn run_list(&self, l_args: &LArgs) -> Result<WekanResult, Error> {
        info!("run_list_command");
        if !l_args.board.is_empty() {
            #[cfg(feature = "store")]
            let mut query = Query {
                config: self.client.config.clone(),
                deny_store_usage: self.global_options.no_store,
            };
            #[cfg(not(feature = "store"))]
            let mut query = Query {
                config: self.client.config.clone(),
            };
            let format = &self.global_options.filter;
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
                    let client = <Client as ListApi>::new(
                        self.client.config.clone(),
                        constraint.board._id.to_owned(),
                    );
                    trace!("{:?}", client);
                    let mut runner: LRunner = LRunner::new(
                        l_args.clone(),
                        client,
                        constraint,
                        self.format.to_owned(),
                        self.display.to_owned(),
                        &self.global_options
                    );
                    runner.apply().await
                }
                Err(e) => Err(e),
            }
        } else {
            WekanResult::new_exit("Board not found in cache or given as argument.", 2, None).ok()
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

        #[cfg(feature = "store")]
        let query = Query {
            config: self.client.config.clone(),
            deny_store_usage: self.global_options.no_store,
        };
        #[cfg(not(feature = "store"))]
        let query = Query {
            config: self.client.config.clone(),
        };
        let client = <Client as CardApi>::new(
            self.client.config.clone(),
            constraint.board.as_ref().unwrap()._id.to_owned(),
            constraint.list.as_ref().unwrap()._id.to_owned(),
        );
        let mut runner: CRunner = CRunner::new(
            c_args.clone(),
            client.clone(),
            constraint,
            self.format.to_owned(),
            query,
            Some(self.global_options.filter.as_ref().unwrap().to_string()),
            self.display.to_owned(),
        );
        runner.run().await
    }

    async fn run_table(&mut self, table_args: &TArgs) -> Result<WekanResult, Error> {
        #[cfg(feature = "store")]
        let mut query = Query {
            config: self.client.config.clone(),
            deny_store_usage: self.global_options.no_store,
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
                    .inquire(AType::List, Some(&board_id), None)
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
                                    .inquire(AType::Card, Some(&board_id), Some(&r.get_id()))
                                    .await
                                {
                                    Ok(cards) => {
                                        trace!("{:?}", cards);
                                        cards_of_lists.push(cards)
                                    }
                                    Err(_e) => cards_of_lists.push(Vec::new()),
                                };
                            }
                            self.display.print_table(lists, cards_of_lists)
                        } else {
                            self.display.print_table(lists, Vec::new())
                        }
                    }
                    Err(_e) => Err(CliError::new_msg("List name doesn't exist.").as_enum()),
                }
            }
            Err(_e) => Err(CliError::new_msg("Board name doesn't exist.").as_enum()),
        }
    }

    async fn run_inspect(&mut self, i: Inspect) -> Result<WekanResult, Error> {
        let mut v: Vec<&str> = i.id.split_terminator('/').collect();
        debug!("describe");
        trace!("Vector: {:?}", v);
        if v.len() != 2 {
            WekanResult::new_msg("Format not correct type/id.").ok()
        } else {
            let id = v.remove(1);
            self.verify_id_length(id.to_string())?;
            match v.remove(0) {
                "board" | "b" => {
                    let mut client = <Client as BoardApi>::new(self.client.config.clone());
                    match client.get_one::<BDetails>(id).await {
                        Ok(b) => self.display.print_details(b, self.global_options.output_format.to_owned()),
                        Err(e) => {
                            error!("Error: {:?}", e);
                            WekanResult::new_msg("Artifact not found.").ok()
                        }
                    }
                }
                "list" | "l" => match i.delegate.board_id {
                    Some(b_id) => {
                        self.verify_id_length(b_id.to_string())?;
                        let mut client =
                            <Client as ListApi>::new(self.client.config.clone(), b_id.to_string());
                        let artifact = client.get_one::<LDetails>(v.remove(0)).await.unwrap();
                        self.display
                            .print_details(artifact, self.global_options.output_format.to_owned())
                    }
                    None => WekanResult::new_msg("Board id needs to be supplied.").ok(),
                },
                "card" | "c" => match i.delegate.board_id {
                    Some(b_id) => {
                        self.verify_id_length(b_id.to_owned())?;
                        match &i.delegate.list_id {
                            Some(l_id) => {
                                self.verify_id_length(l_id.to_string())?;
                                let mut client = <Client as CardApi>::new(
                                    self.client.config.clone(),
                                    b_id.to_string(),
                                    l_id.to_string(),
                                );
                                let artifact =
                                    client.get_one::<CDetails>(v.remove(2)).await.unwrap();
                                self.display
                                    .print_details(artifact, self.global_options.output_format.to_owned())
                            }
                            None => WekanResult::new_msg("List id needs to be supplied.").ok(),
                        }
                    }
                    None => WekanResult::new_msg("Board id needs to be supplied.").ok(),
                },
                _ => WekanResult::new_workflow(
                    "Type does not match.",
                    "Fix your type or look for the resource_type/resource_complete_id combination.",
                )
                .ok(),
            }
        }
    }

    async fn run_describe(&mut self, d: Describe) -> Result<WekanResult, Error> {
        let mut v: Vec<&str> = d.resource.split_terminator('/').collect();
        debug!("describe");
        trace!("Vector: {:?}", v);
        #[cfg(feature = "store")]
        let mut query = Query {
            config: self.client.config.clone(),
            deny_store_usage: self.global_options.no_store,
        };
        #[cfg(not(feature = "store"))]
        let mut query = Query {
            config: self.client.config.clone(),
        };
        let filter = &self.global_options.filter;
        if v.len() != 2 {
            WekanResult::new_msg("Format not correct resource_type/resource_name.").ok()
        } else {
            let name = v.remove(1);
            match v.remove(0) {
                "board" | "b" => {
                    let mut client = <Client as BoardApi>::new(self.client.config.clone());
                    match query.find_board_id(name, filter).await {
                        Ok(board_id) => {
                            let board = client.get_one::<BDetails>(&board_id).await.unwrap();
                            self.display
                                .print_details(board, self.global_options.output_format.to_owned())
                        }
                        Err(_e) => Err(CliError::new_msg("Board name doesn't exist.").as_enum()),
                    }
                }
                "list" | "l" => match &d.delegate.board_id {
                    Some(b_id) => {
                        let mut client =
                            <Client as ListApi>::new(self.client.config.clone(), b_id.to_string());
                        match query.find_list_id(b_id, name, filter).await {
                            Ok(l_id) => {
                                let board = client.get_one::<LDetails>(&l_id).await.unwrap();
                                self.display
                                    .print_details(board, self.global_options.output_format.to_owned())
                            }
                            Err(_e) => {
                                Err(CliError::new_msg("Board name doesn't exist.").as_enum())
                            }
                        }
                    }
                    None => WekanResult::new_msg("Board name needs to be supplied.").ok(),
                },
                "card" | "c" => match &d.delegate.board_id {
                    Some(b_id) => match &d.delegate.list_id {
                        Some(l_id) => {
                            let mut client = <Client as CardApi>::new(
                                self.client.config.clone(),
                                b_id.to_string(),
                                l_id.to_string(),
                            );
                            match query.find_card_id(b_id, l_id, name, filter).await {
                                Ok(c_id) => {
                                    let board = client.get_one::<CDetails>(&c_id).await.unwrap();
                                    self.display
                                        .print_details(board, self.global_options.output_format.to_owned())
                                }
                                Err(_e) => {
                                    Err(CliError::new_msg("Card name doesn't exist.").as_enum())
                                }
                            }
                        }
                        None => WekanResult::new_msg("List name needs to be supplied.").ok(),
                    },
                    None => WekanResult::new_msg("Board name needs to be supplied.").ok(),
                },
                _ => WekanResult::new_workflow(
                    "Type does not match.",
                    "Fix your type or look for the type/id combination.",
                )
                .ok(),
            }
        }
    }

    fn verify_id_length(&self, id: String) -> Result<bool, Error> {
        if id.len() == 17 {
            Ok(true)
        } else {
            Err(InputError::new_msg("Id is not of specified length. Use -o extended to get the complete id of an artifact.").as_enum())
        }
    }
}

#[cfg(feature = "store")]
#[async_trait]
impl ReadContext for UserConfig {
    async fn read_context(&self) -> Result<UserConfig, Error> {
        info!("read_context");
        debug!("{:?}", self.get_path());
        match tokio::fs::read(self.get_path() + "/config").await {
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
