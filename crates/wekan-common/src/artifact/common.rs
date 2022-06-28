use crate::error::ParseError;
use crate::http::artifact::RequestBody;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[cfg(feature = "test")]
use super::tests::{MockDetails, MockResponse};

#[derive(Deserialize, Serialize, Debug, Clone, Eq)]
pub struct Artifact {
    pub _id: String,
    pub title: String,
    #[serde(skip_deserializing)]
    pub r#type: AType,
}
#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct WipLimit {
    value: i8,
    enabled: bool,
    soft: bool,
}

impl WipLimit {
    pub fn new() -> Self {
        Self {
            value: 0,
            enabled: false,
            soft: false,
        }
    }
}

impl Default for WipLimit {
    fn default() -> Self {
        WipLimit::new()
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum AType {
    Empty,
    Board,
    List,
    Card,
    Swimlane,
    Checklist,
}

impl From<String> for AType {
    fn from(s: String) -> Self {
        let fixed = s.as_str();
        match fixed {
            "board" => AType::Board,
            "list" => AType::List,
            "card" | "cardType-card" => AType::Card,
            "swimlane" => AType::Swimlane,
            "artifact" => AType::Empty,
            "template-container" => AType::Empty,
            "checklist" => AType::Checklist,
            _ => panic!("AType for {:?} not implemented.", fixed),
        }
    }
}

impl Default for AType {
    fn default() -> Self {
        AType::Board
    }
}

impl From<AType> for String {
    fn from(artifact: AType) -> Self {
        match artifact {
            AType::Board => "board".to_string(),
            AType::List => "list".to_string(),
            AType::Card => "card".to_string(),
            AType::Swimlane => "swimlane".to_string(),
            AType::Empty => "artifact".to_string(),
            AType::Checklist => "checklist".to_string(),
        }
    }
}

impl ToString for AType {
    fn to_string(&self) -> String {
        match self {
            AType::Board => "board".to_string(),
            AType::List => "list".to_string(),
            AType::Card => "card".to_string(),
            AType::Swimlane => "swimlane".to_string(),
            AType::Empty => "artifact".to_string(),
            AType::Checklist => "checklist".to_string(),
        }
    }
}

impl FromStr for AType {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "board" => Ok(AType::Board),
            "list" => Ok(AType::List),
            "card" | "cardType-card" => Ok(AType::Card),
            "swimlane" => Ok(AType::Swimlane),
            "template-container" => Ok(AType::Empty),
            "checklist" => Ok(AType::Checklist),
            _ => Err(ParseError::new("Not a Wekan kind")),
        }
    }
}
pub trait Base {
    fn get_title(&self) -> String;
    fn set_id(&mut self, id: &str) -> String;
}

pub trait IdReturner {
    fn get_id(&self) -> String;
}

impl IdReturner for Artifact {
    fn get_id(&self) -> String {
        self._id.to_owned()
    }
}
impl Base for Artifact {
    fn get_title(&self) -> String {
        self.title.to_owned()
    }

    fn set_id(&mut self, id: &str) -> String {
        self._id = id.to_owned();
        self._id.to_owned()
    }
}

pub trait BaseDetails {
    fn get_archive_at(&self) -> Option<&String>;
    fn get_modified_at(&self) -> String;
    fn get_created_at(&self) -> String;
}

pub trait MostDetails {
    fn get_description(&self) -> String;
    fn get_due_at(&self) -> String;
    fn get_end_at(&self) -> String;
}
pub trait SortedArtifact {
    fn get_type(&self) -> AType;
    fn get_sort(&self) -> &f32;
    fn set_type(&mut self, t: AType) -> AType;
}

impl SortedArtifact for Artifact {
    fn get_type(&self) -> AType {
        self.r#type.to_owned()
    }
    fn get_sort(&self) -> &f32 {
        &0.0f32
    }

    fn set_type(&mut self, t: AType) -> AType {
        self.r#type = t;
        self.r#type.to_owned()
    }
}
pub trait Elisp {
    fn to_elisp(&self) -> String;
}

impl Elisp for Artifact {
    fn to_elisp(&self) -> String {
        format!("((_id . {}) (title . {})) ", self._id, self.title)
    }
}
impl std::fmt::Display for Artifact {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}    {}", self._id, self.title)
    }
}

pub trait WekanDisplay: SortedArtifact + std::fmt::Debug + Base + IdReturner {}
impl WekanDisplay for Artifact {}
pub trait StoreTrait:
    SortedArtifact + Base + Clone + std::marker::Send + std::marker::Sync + std::fmt::Debug + Serialize
{
}

impl StoreTrait for Artifact {}
impl SortedArtifact for Vec<Artifact> {
    fn get_type(&self) -> AType {
        match self.first() {
            Some(v) => v.get_type(),
            None => {
                if self.is_empty() {
                    AType::Empty
                } else {
                    panic!("Not an artifact variant")
                }
            }
        }
    }
    fn get_sort(&self) -> &f32 {
        &0.0f32
    }

    fn set_type(&mut self, t: AType) -> AType {
        match self.first() {
            Some(v) => {
                if v.get_type() == t {
                    v.get_type()
                } else {
                    panic!("Not an artifact variant")
                }
            }
            None => panic!("Not an artifact variant"),
        }
    }
}
impl Base for Vec<Artifact> {
    fn get_title(&self) -> String {
        match self.first() {
            Some(_v) => "s".to_string(),
            None => panic!("Not an artifact variant"),
        }
    }
    fn set_id(&mut self, _id: &str) -> String {
        self.get_id()
    }
}

impl IdReturner for Vec<Artifact> {
    fn get_id(&self) -> String {
        match self.first() {
            Some(v) => v.get_id(),
            None => panic!("Not an artifact variant"),
        }
    }
}
impl<Artifact: Clone + Base + WekanDisplay + std::marker::Sync + Serialize + std::marker::Send>
    StoreTrait for Vec<Artifact>
where
    Vec<Artifact>: Base + SortedArtifact,
{
}
impl PartialOrd for Artifact {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Artifact {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.title.cmp(&other.title)
    }
}

impl PartialEq for Artifact {
    fn eq(&self, other: &Self) -> bool {
        self.title == other.title
    }
}

impl RequestBody for Artifact {}
#[cfg(feature = "test")]
pub trait DeserializeExt:
    serde::de::DeserializeOwned
    + std::fmt::Debug
    + std::marker::Send
    + 'static
    + super::tests::MockResponse
{
}
#[cfg(not(feature = "test"))]
pub trait DeserializeExt:
    serde::de::DeserializeOwned + std::fmt::Debug + std::marker::Send + 'static
{
}
pub trait WekanDisplayExt: DeserializeExt + StoreTrait + BaseDetails + WekanDisplay {}
pub trait SerializeExt: serde::Serialize + std::fmt::Debug + Clone + std::marker::Send {}

impl DeserializeExt for Artifact {}

#[cfg(feature = "test")]
impl MockDetails for Artifact {
    fn mock(id: &str, title: &str, atype: &str) -> Self {
        Artifact {
            _id: id.to_string(),
            title: title.to_string(),
            r#type: AType::from(atype.to_string()),
        }
    }
}

#[cfg(feature = "test")]
impl MockResponse for Artifact {
    fn mock() -> Self {
        <Self as MockDetails>::mock(
            "fake-artifact-id",
            "fake-artifact-title",
            &AType::Board.to_string(),
        )
    }
}
