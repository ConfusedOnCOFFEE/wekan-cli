use super::common::{
    AType, Base, BaseDetails, DeserializeExt, IdReturner, SortedArtifact, StoreTrait, WekanDisplay,
};
#[cfg(feature = "test")]
use super::tests::{MockDetails, MockResponse};
use crate::http::artifact::RequestBody;
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Deserialize, Debug, PartialEq, Serialize, Clone)]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct Details {
    pub _id: Option<String>,
    pub title: String,
    pub card_id: String,
    pub board_id: Option<String>,
    created_at: String,
    modified_at: String,
    finished_at: String,
    sort: Option<f32>,
    r#type: Option<String>,
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
        None
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
        match &self.r#type {
            Some(t) => AType::from(t.to_owned()),
            None => AType::Empty,
        }
    }

    fn get_sort(&self) -> &f32 {
        match &self.sort {
            Some(a) => a,
            None => &0.0f32,
        }
    }

    fn set_type(&mut self, t: AType) -> AType {
        self.r#type = Some(t.to_string());
        t
    }
}
impl StoreTrait for Details {}
impl RequestBody for Details {}
impl DeserializeExt for Details {}
impl super::common::WekanDisplayExt for Details {}
impl crate::http::artifact::DetailsResponse for Details {}

#[cfg(feature = "test")]
impl MockDetails for Details {
    fn mock(id: &str, title: &str, date: &str) -> Self {
        Self {
            _id: Some(id.to_string()),
            title: title.to_string(),
            board_id: Some(String::from("my-fake-board-id")),
            card_id: String::from("my-fake-card-id"),
            created_at: date.to_string(),
            modified_at: date.to_string(),
            finished_at: date.to_string(),
            sort: None,
            r#type: Some(AType::Checklist.to_string()),
        }
    }
}
impl WekanDisplay for Details {}
#[cfg(feature = "test")]
impl MockResponse for Details {
    fn mock() -> Self {
        <Self as MockDetails>::mock(
            "my-fake-checklist-id",
            "fake-checklist-title",
            "2020-10-12T",
        )
    }
}
