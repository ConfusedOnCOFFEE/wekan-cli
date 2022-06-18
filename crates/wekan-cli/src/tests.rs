use async_trait::async_trait;
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use wekan_common::{
    artifact::common::{AType, Artifact},
    artifact::tests::{MockDetails, MockResponse, MockReturn},
    http::artifact::{Deleted, IdResponse, RequestBody},
};
use wekan_core::{client::Client, error::kind::Error};

pub mod mocks {
    use super::*;
    use wekan_common::validation::authentication::Token;
    use wekan_core::config::{MandatoryConfig, UserConfig};
    #[async_trait]
    pub trait Operation {
        async fn create<
            U: RequestBody,
            T: MockResponse + Send + Debug + DeserializeOwned + 'static,
        >(
            &mut self,
            _body: &U,
        ) -> Result<T, Error> {
            Ok(T::mock())
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
        async fn get_one<T: MockResponse + DeserializeOwned + 'static>(
            &mut self,
            _id: &str,
        ) -> Result<T, Error> {
            Ok(T::mock())
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
            let id_prefix = String::from("fake-");
            let id_suffix = String::from("-id-");
            let title_prefix = String::from("fake-");
            let title_suffix = String::from("-title-");
            vec![
                <Artifact as MockDetails>::mock(
                    &(id_prefix.to_owned() + &t.to_string() + &id_suffix + "1"),
                    &(title_prefix.to_owned() + &t.to_string() + &title_suffix + "1"),
                    &t.to_string(),
                ),
                <Artifact as MockDetails>::mock(
                    &(id_prefix + &t.to_string() + &id_suffix + "2"),
                    &(title_prefix + &t.to_string() + &title_suffix + "2"),
                    &t.to_string(),
                ),
            ]
        }
    }
}
