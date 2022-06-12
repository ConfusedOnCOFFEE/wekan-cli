#[cfg(test)]
mod http {
    use hyper::Response;
    use reqwest::{Accepts, Response, Url};
    use wekan_common::login::Client as LClient;
    use wekan_core::{client::Client, config::UserConfig, http::Client as HClient};

    // impl Response for Response {
    //     fn new() -> Response {
    //         Response {
    //         }
    //     }
    // }
    // #[tokio::test]
    // fn get() {
    //     let userconfig = UserConfig {
    //         url: String::from("url"),
    //         name: None,
    //         token: None
    //     };
    //     let client = Client::new(userconfig);
    //     let res = reqwest::Response::new(hyper::Response::new("new_world"), Url::parse("https://example.net")?, Accepts {}, None);
    //     assert_eq!(client.get(&String::from("url")).await, Ok(res));
    // }

    // #[test]
    // fn set_url_config() {
    //     env::set_var("WEKAN_URL", "yes");
    //     assert_eq!(UserConfig::set_url(), "yes");
    // }

    // #[test]
    // #[should_panic]
    // fn set_url_no_env() {
    //     env::remove_var("WEKAN_URL");
    //     UserConfig::set_url();
    // }

    // #[test]
    // #[should_panic]
    // fn get_user_id_no_id() {
    //     let config = UserConfig {
    //         url: String::from("url"),
    //         name: None,
    //         token: None
    //     };
    //     assert_eq!(config.get_logged_in_user_id(), "123");
    // }

    // #[test]
    // fn get_user_id() {
    //     let config = UserConfig {
    //         url: String::from("url"),
    //         name: None,
    //         token: Some(Token {
    //             id: Box::new(String::from("1")),
    //             token: Box::new(String::from("2")),
    //             token_expires: Box::new(String::from("3"))
    //         })
    //     };
    //     assert_eq!(config.get_logged_in_user_id(), "1");
    // }
}
