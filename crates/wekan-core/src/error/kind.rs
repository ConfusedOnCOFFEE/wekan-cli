use reqwest::Error as ReqError;
use wekan_common::validation::constraint::Constraint;

#[derive(Debug)]
pub enum Error {
    Http(ReqError),
    Constraint(Constraint),
    Io(std::io::Error),
    Yaml(serde_yaml::Error),
    #[cfg(test)]
    UrlParse(url::ParseError),
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

#[cfg(test)]
impl From<url::ParseError> for Error {
    fn from(error: url::ParseError) -> Self {
        Error::UrlParse(error)
    }
}

#[derive(Debug)]
pub struct CoreOk {
    pub name: String,
}
