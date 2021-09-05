use std::{error, fmt::Display};

use super::ConfigFormat;

#[derive(Debug)]
pub enum CargoMetadataError {
    InvalidToml(toml::de::Error),
    NoData,
    CargoTomlIsNotTable,
    InvalidDataStructure(toml::de::Error),
    FoundMultiple,
}

impl Display for CargoMetadataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CargoMetadataError::InvalidToml(err) => write!(f, "invalid toml: {}", err),
            CargoMetadataError::NoData => write!(f, "no field"),
            CargoMetadataError::CargoTomlIsNotTable => write!(f, "Cargo.toml is not table"),
            CargoMetadataError::InvalidDataStructure(err) => write!(f, "invalid data: {}", err),
            CargoMetadataError::FoundMultiple => write!(f, "multiple fields are found"),
        }
    }
}

impl error::Error for CargoMetadataError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            CargoMetadataError::InvalidToml(err)
            | CargoMetadataError::InvalidDataStructure(err) => Some(err),
            _ => None,
        }
    }
}

macro_rules! enum_auto_from {
    ($(#[$a:meta])* $v:vis enum $name:ident { $($e:ident($err:path)),* $(,)? }) => {
        $(#[$a])*
        $v enum $name {
            $( $e($err) ),*
        }

        $(
            impl From<$err> for $name {
                fn from(v: $err) -> Self {
                    Self::$e(v)
                }
            }
        )*
    };
}

enum_auto_from! {
    #[derive(Debug)]
    pub enum ConfigDeserializeErrorKind {
        JsonError(serde_json::Error),
        YamlError(serde_yaml::Error),
        RonError(ron::Error),
        TomlError(toml::de::Error),
        CargoMetadataError(CargoMetadataError),
    }
}

impl Display for ConfigDeserializeErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigDeserializeErrorKind::JsonError(err) => write!(f, "{}", err),
            ConfigDeserializeErrorKind::YamlError(err) => write!(f, "{}", err),
            ConfigDeserializeErrorKind::RonError(err) => write!(f, "{}", err),
            ConfigDeserializeErrorKind::TomlError(err) => write!(f, "{}", err),
            ConfigDeserializeErrorKind::CargoMetadataError(err) => write!(f, "{}", err),
        }
    }
}

impl error::Error for ConfigDeserializeErrorKind {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            ConfigDeserializeErrorKind::JsonError(err) => Some(err),
            ConfigDeserializeErrorKind::YamlError(err) => Some(err),
            ConfigDeserializeErrorKind::RonError(err) => Some(err),
            ConfigDeserializeErrorKind::TomlError(err) => Some(err),
            ConfigDeserializeErrorKind::CargoMetadataError(err) => Some(err),
        }
    }
}

#[non_exhaustive]
#[derive(Debug)]
pub struct ConfigDeserializeError {
    pub filename: String,
    pub format: ConfigFormat,
    pub kind: ConfigDeserializeErrorKind,
}

impl error::Error for ConfigDeserializeError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        self.kind.source()
    }
}

impl Display for ConfigDeserializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid config file {}: {}", self.filename, self.kind)
    }
}
