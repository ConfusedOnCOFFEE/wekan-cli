use super::common::{
    AType, Base, BaseDetails, DeserializeExt, IdReturner, SortedArtifact, StoreTrait, WekanDisplay,
};
use crate::http::artifact::RequestBody;
use serde::{Deserialize, Serialize};
use std::vec::Vec;

#[cfg(feature = "test")]
use crate::artifact::tests::{MockDetails, MockResponse};
#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Label {
    _id: String,
    name: String,
    color: String,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct Member {
    user_id: String,
    is_admin: bool,
    is_active: bool,
    is_no_comments: bool,
    is_comment_only: bool,
    is_worker: bool,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct Org {
    org_id: String,
    org_display_ame: String,
    is_active: bool,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct Team {
    team_id: String,
    team_display_name: String,
    is_active: bool,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct Details {
    #[serde(skip_deserializing)]
    _id: String,
    title: String,
    permission: String,
    slug: String,
    archived: bool,
    archived_at: Option<String>,
    created_at: String,
    modified_at: String,
    stars: f32,
    labels: Option<Vec<Label>>,
    members: Vec<Member>,
    orgs: Option<Vec<Org>>,
    teams: Option<Vec<Team>>,
    color: String,
    description: Option<String>,
    subtasks_default_board_id: Option<String>,
    subtasks_default_list_id: Option<String>,
    date_settings_default_board_id: Option<String>,
    date_settings_default_list_id: Option<String>,
    allows_subtasks: bool,
    allows_attachments: bool,
    allows_checklists: bool,
    allows_comments: bool,
    allows_description_title: bool,
    allows_description_text: bool,
    allows_card_number: bool,
    allows_activities: bool,
    allows_labels: bool,
    allows_creator: bool,
    allows_assignee: bool,
    allows_members: bool,
    allows_requested_by: bool,
    allows_card_sorting_by_number: bool,
    allows_assigned_by: bool,
    allows_received_date: bool,
    allows_start_date: bool,
    allows_end_date: bool,
    allows_due_date: bool,
    present_parent_task: String,
    received_at: Option<String>,
    start_at: Option<String>,
    due_at: Option<String>,
    end_at: Option<String>,
    spent_time: Option<i8>,
    is_overtime: bool,
    r#type: String,
    sort: Option<f32>,
}

trait DetailsSettings {
    fn is_subtasks(&self) -> bool;
    fn is_attachments(&self) -> bool;
    fn is_checklists(&self) -> bool;
    fn is_comments(&self) -> bool;
    fn is_description_title(&self) -> bool;
    fn is_description_text(&self) -> bool;
    fn is_card_number(&self) -> bool;
    fn is_activities(&self) -> bool;
    fn is_labels(&self) -> bool;
    fn is_creator(&self) -> bool;
    fn is_assignee(&self) -> bool;
    fn is_members(&self) -> bool;
    fn is_requested_by(&self) -> bool;
    fn is_card_sorting_by_number(&self) -> bool;
    fn is_assigned_by(&self) -> bool;
    fn is_received_date(&self) -> bool;
    fn is_start_date(&self) -> bool;
    fn is_end_date(&self) -> bool;
    fn is_due_date(&self) -> bool;
}

impl DetailsSettings for Details {
    fn is_subtasks(&self) -> bool {
        self.allows_subtasks.to_owned()
    }
    fn is_attachments(&self) -> bool {
        self.allows_attachments.to_owned()
    }
    fn is_checklists(&self) -> bool {
        self.allows_checklists.to_owned()
    }
    fn is_comments(&self) -> bool {
        self.allows_comments.to_owned()
    }
    fn is_description_title(&self) -> bool {
        self.allows_description_title.to_owned()
    }
    fn is_description_text(&self) -> bool {
        self.allows_description_text.to_owned()
    }
    fn is_card_number(&self) -> bool {
        self.allows_card_number.to_owned()
    }
    fn is_activities(&self) -> bool {
        self.allows_activities.to_owned()
    }
    fn is_labels(&self) -> bool {
        self.allows_labels.to_owned()
    }
    fn is_creator(&self) -> bool {
        self.allows_creator.to_owned()
    }
    fn is_assignee(&self) -> bool {
        self.allows_assignee.to_owned()
    }
    fn is_members(&self) -> bool {
        self.allows_members.to_owned()
    }
    fn is_requested_by(&self) -> bool {
        self.allows_requested_by.to_owned()
    }
    fn is_card_sorting_by_number(&self) -> bool {
        self.allows_card_sorting_by_number.to_owned()
    }
    fn is_assigned_by(&self) -> bool {
        self.allows_assigned_by.to_owned()
    }
    fn is_received_date(&self) -> bool {
        self.allows_received_date.to_owned()
    }
    fn is_start_date(&self) -> bool {
        self.allows_start_date.to_owned()
    }
    fn is_end_date(&self) -> bool {
        self.allows_end_date.to_owned()
    }
    fn is_due_date(&self) -> bool {
        self.allows_due_date.to_owned()
    }
}

impl Base for Details {
    fn get_title(&self) -> String {
        self.title.to_owned()
    }
    fn set_id(&mut self, id: &str) -> String {
        self._id = id.to_owned();
        self._id.to_owned()
    }
}

impl IdReturner for Details {
    fn get_id(&self) -> String {
        self._id.to_owned()
    }
}

impl BaseDetails for Details {
    fn get_archive_at(&self) -> Option<&String> {
        self.archived_at.as_ref()
    }

    fn get_modified_at(&self) -> String {
        self.modified_at.to_owned()
    }

    fn get_created_at(&self) -> String {
        self.created_at.to_owned()
    }
}

impl SortedArtifact for Details {
    fn get_type(&self) -> AType {
        AType::from(self.r#type.to_owned())
    }

    fn get_sort(&self) -> &f32 {
        match &self.sort {
            Some(a) => a,
            None => &0.0f32,
        }
    }

    fn set_type(&mut self, t: AType) -> AType {
        self.r#type = t.to_string();
        AType::from(self.r#type.to_owned())
    }
}
impl StoreTrait for Details {}
impl RequestBody for Details {}
impl WekanDisplay for Details {}
impl DeserializeExt for Details {}

#[cfg(feature = "test")]
impl MockDetails for Details {
    fn mock(id: &str, title: &str, date: &str) -> Self {
        Self {
            _id: id.to_string(),
            title: title.to_string(),
            archived: false,
            archived_at: None,
            color: String::from("red"),
            created_at: date.to_string(),
            modified_at: date.to_string(),
            description: None,
            members: Vec::new(),
            received_at: None,
            start_at: None,
            due_at: Some(date.to_string()),
            end_at: Some(date.to_string()),
            spent_time: None,
            is_overtime: false,
            sort: None,
            r#type: AType::Board.to_string(),
            subtasks_default_board_id: None,
            subtasks_default_list_id: None,
            date_settings_default_board_id: None,
            date_settings_default_list_id: None,
            allows_subtasks: true,
            allows_attachments: true,
            allows_checklists: true,
            allows_comments: true,
            allows_description_title: true,
            allows_description_text: true,
            allows_card_number: true,
            allows_activities: true,
            allows_labels: true,
            allows_creator: true,
            allows_assignee: true,
            allows_members: true,
            allows_requested_by: true,
            allows_card_sorting_by_number: true,
            allows_assigned_by: true,
            allows_received_date: true,
            allows_start_date: true,
            allows_end_date: true,
            allows_due_date: true,
            present_parent_task: String::new(),
            labels: None,
            orgs: None,
            permission: String::new(),
            slug: String::new(),
            stars: 34.0,
            teams: None,
        }
    }
}

#[cfg(feature = "test")]
impl MockResponse for Details {
    fn mock() -> Self {
        <Self as MockDetails>::mock("my-fake-board-id", "fake-board-title", "2020-10-12T")
    }
}
