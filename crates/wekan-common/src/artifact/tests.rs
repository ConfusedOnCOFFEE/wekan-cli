use crate::artifact::common::IdReturner;
use crate::http::artifact::IdResponse;
pub trait MockDetails {
    fn mock(id: &str, title: &str, date: &str) -> Self;
}

pub trait MockResponse {
    fn mock() -> Self;
}

pub trait MockReturn {
    fn success<T: IdResponse>(body: Option<T>) -> Self;
}

impl IdResponse for String {}
impl IdReturner for String {
    fn get_id(&self) -> String {
        self.to_owned()
    }
}
