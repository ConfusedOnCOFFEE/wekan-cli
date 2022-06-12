use async_trait::async_trait;
use clap::Args as ClapArgs;
use log::{debug, info, trace};
use regex::Regex;
use std::str::FromStr;
use wekan_common::validation::authentication::Credentials;
#[cfg(feature = "store")]
use wekan_core::persistence::store::Butler;
use wekan_core::{
    client::LoginClient as Client, config::Setup, http::authentication::TokenManager,
};

use crate::{
    error::kind::{CliError, Error, Transform},
    result::kind::WekanResult,
};
#[cfg(feature = "store")]
#[cfg(not(feature = "integration"))]
use rpassword;
#[cfg(feature = "store")]
use std::fs;

#[derive(ClapArgs, Debug, Clone)]
#[clap(version = "0.1.0", about = "Set credentials.")]
pub struct SetCredentials {
    /// User
    pub user: String,
    #[clap(short = 'd', long, help = "Use only port if you are on localhost.")]
    pub host: String,
    #[clap(
        short = 'i',
        long,
        parse(from_flag),
        help = "Insecure connection. (http)"
    )]
    pub insecure: bool,
}

#[cfg(feature = "store")]
#[derive(ClapArgs, Debug, Copy, Clone)]
#[clap(version = "0.1.0", about = "Delete credentails")]
pub struct DeleteCredentials {}

#[async_trait]
pub trait Authenticate {
    async fn run_login(&mut self, login: &SetCredentials) -> Result<WekanResult, Error>;
    fn password_prompt() -> String;
}
#[async_trait]
impl Authenticate for Client {
    async fn run_login(&mut self, login: &SetCredentials) -> Result<WekanResult, Error> {
        info!("run_login_subcommand");
        let mut splitted_filter: Vec<&str> = login.host.split_terminator(':').collect();
        let mut host = String::from("http");
        if login.insecure {
            host.push_str("://");
        } else {
            host.push_str("s://");
        }

        if splitted_filter.len() == 1 {
            let re = Regex::new(r"^\d*$").unwrap();
            let first = splitted_filter.remove(0);
            if re.is_match(first) {
                debug!("One Input - Port: {:?}", first);
                self.config.set_port(i16::from_str(first).unwrap());
            } else {
                host.push_str("localhost");
                debug!("One-Input Host: {}", host);
                self.config.set_host(host.to_owned());
            }
        };
        if splitted_filter.len() == 2 {
            let p = splitted_filter.remove(1);
            debug!("Port: {:?}", p);
            self.config.set_port(i16::from_str(p).unwrap());
            host.push_str(splitted_filter.remove(0));
            debug!("Host: {}", host);
            self.config.set_host(host);
        };

        let password = Self::password_prompt();
        match self
            .login(Some(Credentials {
                user: login.user.to_owned(),
                pw: password.to_owned(),
            }))
            .await
        {
            Ok(t) => {
                trace!("{:?}", t);
                WekanResult::new_workflow("Login succesful.", "board ls").ok()
            }
            e => {
                println!("{:?}", e);
                CliError::new_msg(
                    "Login unsuccesful. Please check user,pw and if the host is online",
                )
                .err()
            }
        }
    }

    #[cfg(feature = "integration")]
    fn password_prompt() -> String {
        match std::env::var("WEKAN_PWD") {
            Ok(pwd) => pwd,
            Err(_e) => panic!("No password supplied"),
        }
    }
    #[cfg(not(feature = "integration"))]
    fn password_prompt() -> String {
        rpassword::prompt_password("Password: ").unwrap()
    }
}

#[async_trait]
#[cfg(feature = "store")]
pub trait ClearConfig {
    async fn run_logout(&self) -> Result<WekanResult, Error>;
}

#[async_trait]
#[cfg(feature = "store")]
impl ClearConfig for Client {
    async fn run_logout(&self) -> Result<WekanResult, Error> {
        info!("run_logout_subcommand");
        let config_path = self.config.get_path();
        debug!("{}", config_path);
        match fs::remove_file(config_path + "/config") {
            Ok(ok) => {
                debug!("{:?}", ok);
                WekanResult::new_workflow(
                    "Logout succesful. Config deleted.",
                    "'login [USER] [HOST] <POST>",
                )
                .ok()
            }
            Err(e) => Err(Error::Io(e)),
        }
    }
}
