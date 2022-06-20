use async_trait::async_trait;
use log::{debug, info, trace};
use wekan_common::{
    artifact::{
        card::Details,
        common::{AType, Artifact, IdReturner},
    },
    http::{
        artifact::ResponseOk,
        card::{CreateCard, MoveCard, UpdateCard},
    },
    validation::{authentication::TokenHeader, constraint::CardConstraint as Constraint},
};
use wekan_core::client::{CardApi, Client};

use crate::{
    card::argument::{
        Args, CardCreateArgs as Create, CardMoveArgs as Move, Command, RemoveArgs as Remove,
        UpdateArgs as Update,
    },
    command::{CommonRuns, RootCommandRunner, SubCommandRunner},
    display::CliDisplay,
    error::kind::{CliError, Error, Transform},
    resolver::Query,
    result::kind::WekanResult,
    subcommand::Inspect,
};

#[cfg(test)]
use crate::tests::mocks::{Artifacts, Operation};
#[cfg(not(test))]
use wekan_core::http::operation::{Artifacts, Operation};

pub struct Runner {
    pub args: Args,
    pub client: Client,
    pub constraint: Constraint,
    pub format: String,
    pub query: Query,
    pub filter: Option<String>,
    pub display: CliDisplay,
}

pub trait NewCardRunner {
    fn new(
        args: Args,
        client: Client,
        constraint: Constraint,
        format: String,
        query: Query,
        filter: Option<String>,
        display: CliDisplay,
    ) -> Self;
}

impl NewCardRunner for Runner {
    fn new(
        args: Args,
        client: Client,
        constraint: Constraint,
        format: String,
        query: Query,
        filter: Option<String>,
        display: CliDisplay,
    ) -> Self {
        Self {
            args,
            client,
            constraint,
            format,
            query,
            filter,
            display,
        }
    }
}

impl Runner {
    async fn get_details(&mut self, name: &str) -> Result<WekanResult, Error> {
        match self
            .query
            .find_card_id(
                &self.constraint.board.as_ref().unwrap()._id,
                &self.constraint.list.as_ref().unwrap()._id,
                name,
                &self.filter,
            )
            .await
        {
            Ok(card_id) => {
                let card = Artifact {
                    _id: card_id,
                    title: name.to_string(),
                    r#type: AType::Card,
                };
                self.details(&card).await
            }
            Err(_e) => CliError::new_msg("Card doesn't exist.").err(),
        }
    }

    async fn run_create(&mut self, create_args: &Create) -> Result<WekanResult, Error> {
        info!("Create");
        let args = create_args.clone();
        match self
            .query
            .find_swimlane_id(&self.constraint.board.as_ref().unwrap()._id, &self.filter)
            .await
        {
            Ok(swimlane_id) => {
                let create_card = CreateCard {
                    author_id: self.client.get_user_id(),
                    members: None,
                    assignees: None,
                    title: create_args.get_name(),
                    description: args.description,
                    swimlane_id,
                };
                match self
                    .client
                    .create::<CreateCard, ResponseOk>(&create_card)
                    .await
                {
                    Ok(o) => {
                        debug!("CreadCard response");
                        trace!("Created card: {:?}", o);
                        WekanResult::new_workflow(
                            "Successfully created",
                            "Move card or update card",
                        )
                        .ok()
                    }
                    Err(_e) => CliError::new_msg("Card creation failed").err(),
                }
            }
            Err(_e) => CliError::new_msg("List can not be matched to swimlane").err(),
        }
    }

    async fn run_remove(&mut self, remove_args: &Remove) -> Result<WekanResult, Error> {
        info!("Remove");
        match self
            .query
            .find_card_id(
                &self.constraint.board.as_ref().unwrap()._id,
                &self.constraint.list.as_ref().unwrap()._id,
                &remove_args.get_name(),
                &self.filter,
            )
            .await
        {
            Ok(card_id) => {
                debug!("Found card it to remove");
                match self.client.delete::<ResponseOk>(&card_id).await {
                    Ok(_o) => WekanResult::new_msg("Successfully deleted").ok(),
                    Err(e) => {
                        trace!("{:?}", e);
                        CliError::new_msg("Failed to delete").err()
                    }
                }
            }
            Err(e) => Err(e),
        }
    }

    async fn run_move(&mut self, move_args: &Move) -> Result<WekanResult, Error> {
        info!("move");
        debug!("{:?}", move_args.list);
        match self
            .query
            .find_list_id(
                &self.constraint.board.as_ref().unwrap()._id,
                &move_args.list,
                &self.filter,
            )
            .await
        {
            Ok(l_id) => {
                match self
                    .query
                    .find_card_id(
                        &self.constraint.board.as_ref().unwrap()._id,
                        &self.constraint.list.as_ref().unwrap()._id,
                        &move_args.name,
                        &self.filter,
                    )
                    .await
                {
                    Ok(card_id) => {
                        let updated_card = MoveCard { list_id: l_id };
                        match self
                            .client
                            .put::<MoveCard, ResponseOk>(&card_id, &updated_card)
                            .await
                        {
                            Ok(_o) => WekanResult::new_workflow(
                                "Successfully moved",
                                "Update card with more details",
                            )
                            .ok(),
                            Err(_e) => CliError::new_msg("Failed to update").err(),
                        }
                    }
                    Err(_e) => CliError::new_msg("Card couldn't be found").err(),
                }
            }
            _ => CliError::new_msg("List not found").err(),
        }
    }

    async fn run_update(&mut self, update_args: &Update) -> Result<WekanResult, Error> {
        info!("Update");
        match &update_args.card_file {
            Some(_c) => WekanResult::new_workflow("NOT IMPLEMENTED", "Update explicit fields").ok(),
            None => {
                info!("update properties");
                match self
                    .query
                    .find_card_id(
                        &self.constraint.board.as_ref().unwrap()._id,
                        &self.constraint.list.as_ref().unwrap()._id,
                        &update_args.current_name,
                        &self.filter,
                    )
                    .await
                {
                    Ok(card_id) => {
                        let update_card = UpdateCard {
                            title: update_args.title.to_owned(),
                            description: update_args.description.to_owned(),
                            due_at: update_args.due_at.as_ref().map(|d| d.to_string()),
                            end_at: update_args.end_at.as_ref().map(|d| d.to_string()),
                            labels: match &update_args.labels {
                                Some(l) => {
                                    let v: Vec<&str> = l.split_terminator(',').collect();
                                    let new_v: Vec<String> =
                                        v.iter().map(|&s| s.to_string()).collect::<Vec<String>>();
                                    Some(new_v)
                                }
                                None => None,
                            },
                            sort: update_args.sort,
                        };
                        debug!("Update Payload: {:?}", update_card);
                        match self
                            .client
                            .put::<UpdateCard, ResponseOk>(&card_id, &update_card)
                            .await
                        {
                            Ok(_o) => {
                                let card = self.client.get_one::<Details>(&card_id).await.unwrap();
                                self.display.format_most_details(card)
                            }
                            Err(_e) => CliError::new_msg("Failed to update").err(),
                        }
                    }
                    Err(_e) => CliError::new_msg("Card couldn't be found").err(),
                }
            }
        }
    }

    async fn run_details(&mut self) -> Result<WekanResult, Error> {
        match self.args.name.to_owned() {
            Some(n) => self.get_details(&n).await,
            None => CliError::new_msg("Name needs to be supplied.").err(),
        }
    }

    async fn run_inspect(&mut self, inspect: &Inspect) -> Result<WekanResult, Error> {
        info!("Inspect");
        match &inspect.delegate.board_id {
            Some(_b_id) => match &inspect.delegate.list_id {
                Some(_l_id) => {
                    let artifact = self.client.get_one::<Details>(&inspect.id).await.unwrap();
                    self.display
                        .format_base_details(artifact, Some("long".to_string()))
                }
                None => WekanResult::new_msg("List id needs to be supplied.").ok(),
            },
            None => WekanResult::new_msg("Board id needs to be supplied.").ok(),
        }
    }
}

#[async_trait]
impl RootCommandRunner for Runner {
    async fn run(&mut self) -> Result<WekanResult, Error> {
        self.use_subcommand().await
    }
}

#[async_trait]
impl CommonRuns for Runner {
    async fn list<'b>(&mut self, vecs: &'b [Artifact]) -> Result<WekanResult, Error> {
        debug!("list_or_details");
        trace!("{:?}", vecs);
        if !vecs.is_empty() {
            self.display
                .format_vec(vecs.to_vec(), Some(self.format.to_owned()))
        } else {
            debug!("No cards have been found.");
            WekanResult::new_workflow("No cards in the list.", "Card create with 'card -b <BOARD_NAME> -l <LIST_NAME> create [CARD_NAME] --description [CARD_DESCRIPTION]").ok()
        }
    }
    async fn details(&mut self, a: &Artifact) -> Result<WekanResult, Error> {
        let mut client = self.client.clone();
        let card = client.get_one::<Details>(&a.get_id()).await.unwrap();
        trace!("{:?}", card);
        self.display.format_base_details(card, None)
    }
}

#[async_trait]
impl SubCommandRunner for Runner {
    async fn use_subcommand(&mut self) -> Result<WekanResult, Error> {
        let board_name = &self.args.board;
        match self.query.find_board_id(board_name, &self.filter).await {
            Ok(board_id) => {
                debug!("Board id was found");
                trace!("{:?}", board_id);
                trace!("Using filter: {:?}", self.filter);
                self.constraint.board = Some(Artifact {
                    _id: board_id.to_owned(),
                    title: self.constraint.board.as_ref().unwrap().title.to_owned(),
                    r#type: AType::Board,
                });
                match self
                    .query
                    .find_list_id(
                        &self.constraint.board.as_ref().unwrap()._id,
                        &self.args.list,
                        &self.filter,
                    )
                    .await
                {
                    Ok(list_id) => {
                        self.client
                            .set_base(&self.constraint.board.as_ref().unwrap()._id, &list_id);
                        self.constraint.list = Some(Artifact {
                            _id: list_id.to_owned(),
                            title: self.constraint.list.as_ref().unwrap().title.to_owned(),
                            r#type: AType::List,
                        });
                        debug!("List id was found");
                        trace!("{:?}", list_id);
                        match self.args.command.to_owned() {
                            Some(cmd) => match cmd {
                                Command::Create(create_args) => self.run_create(&create_args).await,
                                Command::Remove(remove_args) => self.run_remove(&remove_args).await,
                                Command::Move(move_args) => self.run_move(&move_args).await,
                                Command::Update(update_args) => self.run_update(&update_args).await,
                                Command::Details(_dt) => self.run_details().await,
                                Command::Inspect(i) => self.run_inspect(&i).await,
                            },
                            None => self.run_details().await,
                        }
                    }
                    Err(_e) => CliError::new_msg("No list match.").err(),
                }
            }
            Err(_e) => CliError::new_msg("No card match.").err(),
        }
    }
}
