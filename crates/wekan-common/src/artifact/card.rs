use super::common::{
    AType, Base, BaseDetails, DeserializeExt, IdReturner, MostDetails, SortedArtifact, StoreTrait,
    WekanDisplay,
};
#[cfg(feature = "test")]
use super::tests::{MockDetails, MockResponse};
use crate::http::artifact::RequestBody;
use serde::{Deserialize, Serialize};
use std::vec::Vec;

#[allow(dead_code)]
#[derive(Deserialize, Debug, PartialEq, Serialize, Clone)]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct Vote {
    question: String,
    positive: Vec<String>,
    negative: Vec<String>,
    end: Option<String>,
    public: bool,
    allow_non_board_members: bool,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, PartialEq, Serialize, Clone)]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct Poker {
    question: bool,
    one: Vec<String>,
    two: Vec<String>,
    three: Vec<String>,
    five: Vec<String>,
    eight: Vec<String>,
    thirteen: Vec<String>,
    twenty: Vec<String>,
    forty: Vec<String>,
    one_hundred: Vec<String>,
    unsure: Vec<String>,
    end: Option<String>,
    allow_non_board_members: bool,
    estimation: Option<f64>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, PartialEq, Serialize, Clone)]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct Details {
    pub _id: Option<String>,
    pub title: String,
    archived: Option<bool>,
    archived_at: Option<String>,
    parent_id: Option<String>,
    pub list_id: String,
    swimlane_id: String,
    pub board_id: String,
    cover_id: Option<String>,
    color: Option<String>,
    created_at: String,
    modified_at: String,
    custom_fields: Option<Vec<String>>,
    date_last_activity: String,
    pub description: Option<String>,
    requested_by: Option<String>,
    pub assigned_by: Option<String>,
    label_ids: Option<Vec<String>>,
    members: Option<Vec<String>>,
    pub assignees: Option<Vec<String>>,
    received_at: Option<String>,
    start_at: Option<String>,
    due_at: Option<String>,
    pub end_at: Option<String>,
    spent_time: Option<f64>,
    is_overtime: Option<bool>,
    user_id: String,
    sort: Option<f32>,
    subtask_sort: Option<f64>,
    r#type: String,
    linked_id: Option<String>,
    vote: Option<Vote>,
    poker: Option<Poker>,
    target_id_gantt: Option<Vec<String>>,
    link_type_gantt: Option<Vec<f64>>,
    link_id_gantt: Option<Vec<String>>,
    card_number: Option<f64>,
}

impl Details {}
impl Base for Details {
    fn get_title(&self) -> String {
        self.title.to_owned()
    }
    fn set_id(&mut self, id: &str) -> String {
        self._id = Some(id.to_owned());
        self._id.as_ref().unwrap().to_owned()
    }
}

impl IdReturner for Details {
    fn get_id(&self) -> String {
        match &self._id {
            Some(i) => i.to_owned(),
            None => String::from("XXXXX"),
        }
    }
}

impl BaseDetails for Details {
    fn get_archive_at(&self) -> Option<&String> {
        match &self.archived_at {
            Some(a) => Some(a),
            None => None,
        }
    }

    fn get_modified_at(&self) -> String {
        self.modified_at.to_owned()
    }

    fn get_created_at(&self) -> String {
        self.created_at.to_owned()
    }
}

impl MostDetails for Details {
    fn get_description(&self) -> String {
        match &self.description {
            Some(d) => d.to_owned(),
            None => String::new(),
        }
    }

    fn get_due_at(&self) -> String {
        match &self.due_at {
            Some(d) => d.to_owned(),
            None => String::new(),
        }
    }
    fn get_end_at(&self) -> String {
        match &self.end_at {
            Some(d) => d.to_owned(),
            None => String::new(),
        }
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
impl DeserializeExt for Details {}
#[cfg(feature = "test")]
impl MockDetails for Details {
    fn mock(id: &str, title: &str, date: &str) -> Self {
        Self {
            _id: Some(id.to_string()),
            title: title.to_string(),
            archived: None,
            archived_at: None,
            parent_id: None,
            list_id: String::from("my-fake-card-id"),
            swimlane_id: String::from("my-fake-swimlane-id"),
            board_id: String::from("my-fake-board-id"),
            cover_id: None,
            color: None,
            created_at: date.to_string(),
            modified_at: date.to_string(),
            custom_fields: None,
            date_last_activity: String::new(),
            description: None,
            requested_by: None,
            assigned_by: None,
            label_ids: None,
            members: None,
            assignees: None,
            received_at: None,
            start_at: None,
            due_at: Some(date.to_string()),
            end_at: Some(date.to_string()),
            spent_time: None,
            is_overtime: None,
            user_id: String::new(),
            sort: None,
            subtask_sort: None,
            r#type: AType::Card.to_string(),
            linked_id: None,
            vote: None,
            poker: None,
            target_id_gantt: None,
            link_type_gantt: None,
            link_id_gantt: None,
            card_number: None,
        }
    }
}
impl WekanDisplay for Details {}
#[cfg(feature = "test")]
impl MockResponse for Details {
    fn mock() -> Self {
        <Self as MockDetails>::mock("my-fake-card-id", "fake-card-title", "2020-10-12T")
    }
}
