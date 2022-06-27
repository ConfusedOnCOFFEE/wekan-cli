use crate::{
    card::argument::{Args, CardMoveArgs as Move, Command, UpdateArgs as Update},
    command::{
        Args as RArgs, ArgumentRequester, ArtifactName, CreateSubcommand, Fulfillment, Operator,
        RootCommandRunner,
    },
    display::CliDisplay,
    error::kind::{CliError, Error, Transform},
    resolver::Query,
    result::kind::WekanResult,
    subcommand::{Archive, Inspect},
};
use async_trait::async_trait;
use chrono::{SecondsFormat, Utc};
use log::{info, trace};
use wekan_common::{
    artifact::{card::Details, common::AType},
    http::{
        artifact::ResponseOk,
        card::{ArchiveCard, CreateCard, MoveCard, UpdateCard},
    },
    validation::{authentication::TokenHeader, constraint::CardConstraint as Constraint},
};
use wekan_core::client::{CardApi, Client};

#[cfg(test)]
use crate::tests::mocks::{Artifacts, Operation};
use wekan_cli_derive::FulfilmentRunner;
#[cfg(not(test))]
use wekan_core::http::operation::{Artifacts, Operation};

#[derive(FulfilmentRunner)]
pub struct Runner<'a> {
    pub args: Args,
    pub client: Client,
    pub constraint: Constraint,
    pub query: &'a mut Query<'a>,
    pub format: String,
    pub display: CliDisplay,
    pub global_options: &'a RArgs,
}

#[async_trait]
impl<'a> RootCommandRunner<'a, Details, Command> for Runner<'a> {
    async fn use_specific_command(&mut self) -> Result<WekanResult, Error> {
        info!("use_specific_command");
        match self.args.command.to_owned() {
            Some(c) => match c {
                Command::Update(u) => self.run_update(&u).await,
                Command::Move(m) => self.run_move(&m).await,
                Command::Create(c) => self.use_create(&c).await,
                Command::Archive(a) => self.run_archive(&a).await,
                _ => self.use_common_command().await,
            },
            None => CliError::new_msg("Subcommand not implemented").err(),
        }
    }
    async fn use_ls(&mut self) -> Result<WekanResult, Error> {
        self.get_all().await
    }
    async fn use_create(
        &mut self,
        create_args: &impl CreateSubcommand,
    ) -> Result<WekanResult, Error> {
        match self
            .query
            .find_swimlane_id(&self.constraint.board._id)
            .await
        {
            Ok(swimlane_id) => {
                let create_card = CreateCard {
                    _id: String::new(),
                    author_id: self.client.get_user_id(),
                    members: None,
                    assignees: None,
                    title: create_args.get_title(),
                    description: create_args.get_description(),
                    swimlane_id,
                };
                match self
                    .client
                    .create::<CreateCard, ResponseOk>(&create_card)
                    .await
                {
                    Ok(_o) => WekanResult::new_workflow(
                        "Successfully created",
                        "Move card or update card",
                    )
                    .ok(),
                    Err(_e) => CliError::new_msg("Failed to create").err(),
                }
            }
            Err(_e) => CliError::new_msg("List can not be matched to swimlane").err(),
        }
    }
    async fn use_inspect(&mut self, inspect: &Inspect) -> Result<WekanResult, Error> {
        match &inspect.delegate.board_id {
            Some(b_id) => match &inspect.delegate.list_id {
                Some(l_id) => {
                    self.client.set_base(b_id, l_id);
                    self.get_one::<Details>(&inspect.id).await
                }
                None => WekanResult::new_msg("List id needs to be supplied").ok(),
            },
            None => WekanResult::new_msg("Board id needs to be supplied").ok(),
        }
    }
}

#[async_trait]
impl<'a> Operator<'a> for Runner<'a> {
    async fn find_details_id(&mut self, name: &str) -> Result<String, Error> {
        self.query
            .find_card_id(&self.constraint.board._id, &self.constraint.list._id, name)
            .await
    }
    fn get_type(&self) -> AType {
        AType::Card
    }

    fn get_children_type(&self) -> AType {
        AType::Empty
    }
}

impl<'a> Runner<'a> {
    pub fn new(
        args: Args,
        client: Client,
        constraint: Constraint,
        query: &'a mut Query<'a>,
        format: String,
        display: CliDisplay,
        global_options: &'a RArgs,
    ) -> Self {
        Self {
            args,
            client,
            constraint,
            query,
            format,
            display,
            global_options,
        }
    }
    async fn run_move(&mut self, move_args: &Move) -> Result<WekanResult, Error> {
        info!("run_move");
        match self
            .query
            .find_list_id(&self.constraint.board._id, &move_args.list)
            .await
        {
            Ok(l_id) => {
                trace!("Found destination list id: {}", l_id);
                let name = self.args.get_name()?;
                let id = self.find_details_id(&name).await?;
                let updated_card = MoveCard {
                    list_id: l_id,
                    _id: id,
                };
                match self.client.put::<MoveCard, ResponseOk>(&updated_card).await {
                    Ok(_o) => WekanResult::new_workflow(
                        "Successfully moved",
                        "Update card with more details",
                    )
                    .ok(),
                    Err(_e) => CliError::new_msg("Failed to update").err(),
                }
            }
            _ => CliError::new_msg("Failed to find destination").err(),
        }
    }

    async fn run_update(&mut self, update_args: &Update) -> Result<WekanResult, Error> {
        info!("run_update");
        let name = self.args.get_name()?;
        match self.find_details_id(&name).await {
            Ok(id) => {
                let update_card = UpdateCard {
                    _id: id.to_owned(),
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
                match self
                    .client
                    .put::<UpdateCard, ResponseOk>(&update_card)
                    .await
                {
                    Ok(_o) => {
                        let card = self.client.get_one::<Details>(&id).await.unwrap();
                        self.display.format_most_details(card)
                    }
                    Err(_e) => CliError::new_msg("Failed to update").err(),
                }
            }
            Err(_e) => CliError::new_msg("Failed to find card").err(),
        }
    }

    async fn run_archive(&mut self, archive_args: &Archive) -> Result<WekanResult, Error> {
        info!("use_archive");
        // https://github.com/wekan/wekan/issues/3250
        let name = self.args.get_name()?;
        let id = self.find_details_id(&name).await?;
        let mut archive_card = ArchiveCard {
            _id: id,
            archive: true,
            archive_at: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
        };
        if archive_args.restore {
            archive_card.archive = false;
            archive_card.archive_at = String::new();
        }
        trace!("{:?}", archive_card);
        self.use_archive::<ArchiveCard, Details>(&archive_card)
            .await
    }
}
