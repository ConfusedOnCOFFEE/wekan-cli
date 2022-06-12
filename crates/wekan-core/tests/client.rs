#[cfg(test)]
mod client {
    use wekan_core::client::Client;
    use wekan_core::config::UserConfig;
    #[test]
    fn new_client() {
        let userconfig = UserConfig {
            url: String::from("url"),
            name: None,
            token: None,
        };
        let client = Client::new(userconfig);
        assert_eq!(client.get_api_url(), "url/api/");
    }

    #[tokio::test]
    async fn new_vec() {
        let res = Client::get_result(Ok(Vec::new()));
        assert_eq!(res.await.len(), 0);
    }
}
