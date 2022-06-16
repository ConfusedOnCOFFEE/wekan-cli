mod config {
    #[cfg(feature = "store")]
    use wekan_common::validation::authentication::Token;
    #[cfg(feature = "store")]
    use wekan_core::config::ConfigRequester;
    use wekan_core::config::{AddressConfig, MandatoryConfig, UserConfig};
    #[test]
    fn new_config() {
        let config = UserConfig::new();
        assert_eq!(config.get_address(), "http://localhost:8080");
    }

    #[test]
    #[cfg(feature = "store")]
    fn get_user_id_no_id() {
        let mut config = UserConfig::new();
        config.set_token(Token {
            id: Box::new(String::from("123")),
            token: Box::new(String::from("yNa1VR1Cz6nTzNirWPm2dRNYjdu-EM6LxKDIT0pIYsi")),
            token_expires: Box::new(String::from("2022-08-30T19:37:47.170Z")),
        });
        assert_eq!(config.get_base_id(), "123");
    }
}
