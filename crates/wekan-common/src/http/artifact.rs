use crate::artifact::common::{Base, DeserializeExt, IdReturner};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Debug;

#[cfg(feature = "test")]
use crate::artifact::tests::{MockNewResponse, MockReturn};
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

pub trait RequestBody: std::marker::Sync + Debug + Serialize + Send {}
pub trait Deleted: Debug + DeserializeOwned + 'static {}
pub trait IdResponse: Send + Debug + IdReturner + DeserializeOwned + 'static {}

impl RequestBody for CreateArtifact {}

impl Deleted for ResponseOk {}
impl RequestBody for ResponseOk {}
impl IdResponse for ResponseOk {}
impl DeserializeExt for ResponseOk {}
impl IdReturner for ResponseOk {
    fn get_id(&self) -> String {
        self._id.to_owned()
    }
}
#[cfg(feature = "test")]
impl MockReturn for ResponseOk {
    fn success<T: IdResponse>(body: Option<T>) -> Self {
        match body {
            Some(b) => ResponseOk { _id: b.get_id() },
            None => ResponseOk {
                _id: String::from("fake-id"),
            },
        }
    }
}

impl<T: Clone + Base + std::marker::Sync + Serialize + DeserializeExt + std::marker::Send>
    DeserializeExt for Response<T>
{
}
#[cfg(feature = "test")]
impl<T: DeserializeOwned> MockNewResponse for Response<T> {
    fn new() -> Self {
        Response { array: Vec::new() }
    }
}
#[cfg(feature = "test")]
impl MockNewResponse for ResponseOk {
    fn new() -> Self {
        ResponseOk {
            _id: String::from("fake-id"),
        }
    }
}
