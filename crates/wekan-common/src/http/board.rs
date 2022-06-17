use crate::http::artifact::RequestBody;
use serde::{Deserialize, Serialize};

#[cfg(feature = "test")]
use crate::artifact::tests::MockNewResponse;

#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone, Serialize)]
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
#[serde(tag = "type", rename_all = "camelCase")]
pub struct CreatedBoard {
    #[serde(skip_deserializing)]
    _id: String,
    #[serde(skip_deserializing)]
    default_swimmlane_id: String,
}

impl RequestBody for CreateBoard {}
impl crate::artifact::common::DeserializeExt for CreatedBoard {}
#[cfg(feature = "test")]
impl MockNewResponse for CreatedBoard {
    fn new() -> Self {
        Self {
            _id: String::from("fake-id"),
            default_swimmlane_id: String::from("default"),
        }
    }
}
