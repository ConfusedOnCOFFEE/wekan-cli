use crate::http::artifact::RequestBody;
use serde::{Deserialize, Serialize};

#[cfg(feature = "test")]
use crate::artifact::tests::MockResponse;

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct CreateBoard {
    pub title: String,
    pub owner: String,
    pub permission: Option<String>,
    pub color: Option<String>,
    pub is_admin: Option<bool>,
    pub is_active: Option<bool>,
    pub is_no_comments: Option<bool>,
    pub is_comment_only: Option<bool>,
    pub is_worker: Option<bool>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, Serialize)]
#[serde(tag = "type")]
pub struct CreatedBoard {
    _id: String,
    #[serde(alias = "defaultSwimlaneId")]
    default_swimlane_id: String,
}

impl RequestBody for CreateBoard {}
impl crate::artifact::common::DeserializeExt for CreatedBoard {}
#[cfg(feature = "test")]
impl MockResponse for CreatedBoard {
    fn mock() -> Self {
        Self {
            _id: String::from("fake-return-id"),
            default_swimlane_id: String::from("default"),
        }
    }
}
