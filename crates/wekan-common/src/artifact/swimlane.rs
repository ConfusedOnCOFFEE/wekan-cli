use serde::{Deserialize, Serialize};

use super::common::{
    AType, Base, DeserializeExt, IdReturner, SortedArtifact, StoreTrait, WekanDisplay,
};

#[cfg(feature = "test")]
use crate::artifact::tests::MockNewResponse;

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
impl MockNewResponse for Details {
    fn new() -> Self {
        Details {
            _id: String::from("fake-id"),
            title: None,
            archived: false,
            archived_at: String::new(),
            board_id: String::from("fake-board-id"),
            created_at: String::new(),
            sort: 9,
            updated_at: String::new(),
            color: String::new(),
            r#type: AType::Swimlane.to_string(),
        }
    }
}
