use serde::Deserialize;

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct Details {
    card_id: String,
    checklist_id: String,
    pub title: String,
    is_finished: bool,
    created_at: Option<String>,
    modified_at: String,
    sort: i8,
}
