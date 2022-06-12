use serde::{Deserialize, Serialize};
use super::common::{Base, AType, BaseDetails, SortedArtifact, WipLimit, StoreTrait};

#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct Details {
    #[serde(skip_deserializing)]
    _id: String,
    title: Option<String>,
    starred: bool,
    archived: bool,
    #[serde(default)]
    archived_at: String,
    board_id: String,
    pub swimlane_id: String,
    created_at: String,
    sort: f32,
    updated_at: String,
    modified_at: String,
    wip_limit: WipLimit,
    #[serde(default)]
    color: String,
    r#type: String,
}

impl Base for Details {
    fn get_title(&self) -> String {
        self.title.as_ref().unwrap().to_owned()
    }
    fn get_id(&self) -> String {
        self._id.to_owned()
    }
    fn set_id(&mut self, id: &str) -> String {
        self._id = id.to_owned();
        self._id.to_owned()
    }
}

impl BaseDetails for Details {
    fn get_archive_at(&self) -> Option<&String> {
        Some(&self.archived_at)
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
        &self.sort
    }

    fn set_type(&mut self, t: AType) -> AType {
        self.r#type = t.to_string();
        AType::from(self.r#type.to_owned())
    }
}
impl StoreTrait for Details {}
