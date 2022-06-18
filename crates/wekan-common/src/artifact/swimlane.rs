use serde::{Deserialize, Serialize};

use super::common::{
    AType, Base, DeserializeExt, IdReturner, SortedArtifact, StoreTrait, WekanDisplay,
};

#[cfg(feature = "test")]
use crate::artifact::tests::{MockResponse, MockDetails};

#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct Details {
    #[serde(skip_deserializing)]
    _id: String,
    title: Option<String>,
    archived: bool,
    #[serde(default)]
    archived_at: String,
    board_id: String,
    created_at: String,
    sort: i8,
    updated_at: String,
    #[serde(default)]
    color: String,
    r#type: String,
}

impl Base for Details {
    fn get_title(&self) -> String {
        self.title.as_ref().unwrap().to_owned()
    }
    fn set_id(&mut self, id: &str) -> String {
        self._id = id.to_owned();
        self._id.to_owned()
    }
}

impl IdReturner for Details {
    fn get_id(&self) -> String {
        String::new()
    }
}

impl SortedArtifact for Details {
    fn get_type(&self) -> AType {
        AType::from(self.r#type.to_owned())
    }
    fn get_sort(&self) -> &f32 {
        &0.0f32
    }

    fn set_type(&mut self, t: AType) -> AType {
        self.r#type = t.to_string();
        AType::from(self.r#type.to_owned())
    }
}
impl StoreTrait for Details {}
impl WekanDisplay for Details {}
impl DeserializeExt for Details {}
#[cfg(feature = "test")]
impl MockDetails for Details {
    fn mock(id: &str, title: &str, date: &str) -> Self {
        Self {
            _id: id.to_string(),
            title: Some(title.to_string()),
            archived: false,
            archived_at: date.to_string(),
            board_id: String::from("my-fake-board-id"),
            created_at: date.to_string(),
            sort: 9,
            updated_at: date.to_string(),
            color: String::new(),
            r#type: AType::Swimlane.to_string(),
        }
    }
}

#[cfg(feature = "test")]
impl MockResponse for Details {
    fn mock() -> Self {
        <Self as MockDetails>::mock("my-fake-swimlane-id", "fake-swimlane-title", "2020-10-12T")
    }
}
