use crate::http::artifact::RequestBody;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct CreateCard {
    pub author_id: String,
    pub members: Option<String>,
    pub assignees: Option<String>,
    pub title: String,
    pub description: String,
    pub swimlane_id: String,
}
impl RequestBody for CreateCard {}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct Card {
    description: String,
}

#[allow(dead_code)]
impl Card {
    fn get_description(&self) -> String {
        self.description.to_owned()
    }
}

#[derive(Serialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "camelCase")]
#[allow(dead_code)]
pub struct MoveCard {
    pub list_id: String,
}
impl RequestBody for MoveCard {}
#[allow(dead_code)]
#[derive(Deserialize, Debug, PartialEq, Serialize, Clone)]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct UpdateCard {
    pub title: Option<String>,
    pub description: Option<String>,
    pub due_at: Option<String>,
    pub end_at: Option<String>,
    pub sort: Option<f32>,
    pub labels: Option<Vec<String>>,
}
impl RequestBody for UpdateCard {}
