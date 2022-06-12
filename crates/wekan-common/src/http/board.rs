use serde::{Deserialize, Serialize};

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
    // #[serde(alias = "id")]
    #[serde(skip_deserializing)]
    _id: String,
    // #[serde(alias = "defaultSwimmlaneId")]
    #[serde(skip_deserializing)]
    default_swimmlane_id: String,
}
