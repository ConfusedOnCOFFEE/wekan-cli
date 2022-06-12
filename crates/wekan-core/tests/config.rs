#[cfg(test)]
mod client {
    use std::env;
    use wekan_common::login::Token;
    use wekan_core::config::{MandatoryConfig, UserConfig};
    #[test]
    fn new_config() {
        let config = UserConfig {
            url: String::from("url"),
            name: None,
            token: None,
        };
        assert_eq!(config.get_url(), "url");
    }

    #[test]
    fn set_url_config() {
        env::set_var("WEKAN_URL", "yes");
        assert_eq!(UserConfig::set_url(), "yes");
    }

    #[test]
    #[should_panic]
    fn set_url_no_env() {
        env::remove_var("WEKAN_URL");
        UserConfig::set_url();
    }

    #[test]
    #[should_panic]
    fn get_user_id_no_id() {
        let config = UserConfig {
            url: String::from("url"),
            name: None,
            token: None,
        };
        assert_eq!(config.get_logged_in_user_id(), "123");
    }

    #[test]
    fn get_user_id() {
        let config = UserConfig {
            url: String::from("url"),
            name: None,
            token: Some(Token {
                id: Box::new(String::from("1")),
                token: Box::new(String::from("2")),
                token_expires: Box::new(String::from("3")),
            }),
        };
        assert_eq!(config.get_logged_in_user_id(), "1");
    }
}
