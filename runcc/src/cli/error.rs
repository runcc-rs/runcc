use std::fmt::Display;

use crate::read::error::ConfigDeserializeError;

#[derive(Debug)]
pub enum OptionsError {
    ConfigFileError(ConfigDeserializeError),
    EnvSyntaxError(String),
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
        }
    }
}
