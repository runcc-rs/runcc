use std::fmt::Display;

use crate::read::error::FindConfigError;

#[derive(Debug)]
pub enum OptionsError {
    ConfigFileError(FindConfigError),
    EnvSyntaxError(String),
    DuplicateConfigs,
    NoConfigs,
}

impl std::error::Error for OptionsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            OptionsError::ConfigFileError(err) => Some(err),
            _ => None,
        }
    }
}

impl Display for OptionsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OptionsError::ConfigFileError(err) => write!(f, "Config file error: {}", err),
            OptionsError::EnvSyntaxError(env) => {
                write!(f, "The following env var has invalid syntax: {}", env)
            }
            OptionsError::DuplicateConfigs => {
                write!(
                    f,
                    "Positional arguments and -c option can not be both specified"
                )
            }
            OptionsError::NoConfigs => {
                write!(
                    f,
                    "Please specify commands from config file or positional arguments"
                )
            }
        }
    }
}
