use reqwest::Error as ReqError;
use wekan_common::validation::constraint::Constraint;

#[derive(Debug)]
pub enum Error {
    Http(ReqError),
    Constraint(Constraint),
    Io(std::io::Error),
    Yaml(serde_yaml::Error),
}

impl From<ReqError> for Error {
    fn from(error: ReqError) -> Self {
        Error::Http(error)
    }
}
impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::Io(error)
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(error: serde_yaml::Error) -> Self {
        Error::Yaml(error)
    }
}

#[derive(Debug)]
pub struct CoreOk {
    pub name: String,
}
