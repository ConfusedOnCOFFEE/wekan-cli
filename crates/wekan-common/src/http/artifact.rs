use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Clone)]
#[serde(transparent)]
pub struct Response<T> {
    pub array: Vec<T>,
}
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CreateArtifact {
    pub title: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ResponseOk {
    pub _id: String,
}
