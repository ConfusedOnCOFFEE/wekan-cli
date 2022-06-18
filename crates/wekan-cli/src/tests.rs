use async_trait::async_trait;
use wekan_core::{
    error::kind::Error,
    client::Client
};
use std::fmt::Debug;
use serde::de::DeserializeOwned;
use wekan_common::{
    artifact::common::{AType, Artifact},
    artifact::tests::{MockNewResponse, MockReturn},
    http::artifact::{Deleted, IdResponse, RequestBody},
};

pub mod mocks {
    use super::*;
    use wekan_common::validation::authentication::Token;
    use wekan_core::config::{MandatoryConfig, UserConfig};
    #[async_trait]
    pub trait Operation {
        async fn create<
            U: RequestBody,
            T: MockNewResponse + Send + Debug + DeserializeOwned + 'static,
        >(
            &mut self,
            _body: &U,
        ) -> Result<T, Error> {
            Ok(T::new())
        }
        async fn delete<T: Deleted + MockReturn + IdResponse>(
            &mut self,
            id: &str,
        ) -> Result<T, Error> {
            Ok(T::success(Some(id.to_string())))
        }
        async fn put<U: RequestBody, T: MockReturn + IdResponse>(
            &mut self,
            id: &str,
            _body: &U,
        ) -> Result<T, Error> {
            Ok(T::success(Some(id.to_string())))
        }
    }
    impl Operation for Client {}

    #[async_trait]
    pub trait Artifacts {
        async fn get_all(&mut self, t: AType) -> Result<Vec<Artifact>, Error> {
            Ok(Vec::mocks(t))
        }
        async fn get_one<T: MockNewResponse + DeserializeOwned + 'static>(
            &mut self,
            _id: &str,
        ) -> Result<T, Error> {
            Ok(T::new())
        }
    }
    impl Artifacts for Client {}

    pub trait Mock {
        fn mock() -> Self;
    }
    impl Mock for Client {
        fn mock() -> Self {
            <Client as wekan_core::client::BoardApi>::new(UserConfig::mock())
        }
    }

    impl Mock for Token {
        fn mock() -> Self {
            Token {
               id: Box::new(String::from("123")),
               token: Box::new(String::from("yNa1VR1Cz6nTzNirWPm2dRNYjdu-EM6LxKDIT0pIYsi")),
               token_expires: Box::new(String::from("2022-08-30T19:37:47.170Z")),
            }
        }
    }
    impl Mock for UserConfig {
        fn mock() -> Self {
            let mut config = UserConfig::new();
            config.set_token(Token::mock());
            config
        }
    }

    pub trait Mocks {
        fn mocks(t: AType) -> Self;
    }
    impl Mocks for Vec<Artifact> {
        fn mocks(t: AType) -> Self {
            vec![
                Artifact::new("fake-id-1", "fake-title", t.clone()),
                Artifact::new("fake-id-2", "fake-title2", t),
            ]
        }
    }
}
