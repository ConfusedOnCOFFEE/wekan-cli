use crate::result::kind::WekanResult;
use wekan_common::validation::constraint::Constraint;
use wekan_core::error::kind::Error as CoreError;

#[derive(Debug)]
pub enum Error {
    Core(CoreError),
    Cli(CliError),
    Input(InputError),
    Io(std::io::Error),
    Yaml(serde_yaml::Error),
    #[cfg(feature = "store")]
    Store(StoreError),
}

#[cfg(feature = "store")]
#[derive(Debug, Clone)]
pub struct StoreError {
    pub found: bool,
}

#[derive(Debug, Clone)]
pub struct InputError {
    pub message: String,
}
#[derive(Debug, Clone)]
pub struct CliError {
    pub error_code: i8,
    pub message: String,
    pub constraint: Option<Constraint>,
}

impl CliError {
    pub fn new_constraint(msg: &str, constraint: Constraint) -> Self {
        Self {
            message: msg.to_string(),
            constraint: Some(constraint),
            error_code: 1,
        }
    }
    pub fn new(code: i8, message: &str, constraint: Constraint) -> Self {
        CliError {
            message: message.to_string(),
            constraint: Some(constraint),
            error_code: code,
        }
    }
}
pub trait Transform {
    fn new_msg(msg: &str) -> Self;
    fn as_enum(&self) -> Error;
    fn err(&self) -> Result<WekanResult, Error>;
}

impl Transform for CliError {
    fn new_msg(msg: &str) -> Self {
        Self {
            message: msg.to_string(),
            constraint: None,
            error_code: 1,
        }
    }

    fn as_enum(&self) -> Error {
        Error::Cli(self.to_owned())
    }

    fn err(&self) -> Result<WekanResult, Error> {
        Err(Error::Cli(self.to_owned()))
    }
}

impl Transform for InputError {
    fn new_msg(msg: &str) -> Self {
        Self {
            message: msg.to_string(),
        }
    }
    fn as_enum(&self) -> Error {
        Error::Input(self.to_owned())
    }

    fn err(&self) -> Result<WekanResult, Error> {
        Err(Error::Input(self.to_owned()))
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
impl From<wekan_core::error::kind::Error> for Error {
    fn from(error: wekan_core::error::kind::Error) -> Self {
        Error::Core(error)
    }
}
