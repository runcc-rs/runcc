use serde::de::DeserializeOwned;
use std::{fs, fs::File, str::FromStr};

use super::error::*;

#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub enum ConfigFormat {
    Json,
    Yaml,
    Ron,
    Toml,
    /// See: [Cargo.toml metadata table](https://doc.rust-lang.org/cargo/reference/manifest.html#the-metadata-table)
    CargoMetadata,
}

const EXTENSIONS: [(ConfigFormat, &str); 5] = [
    (ConfigFormat::Json, ".json"),
    (ConfigFormat::Yaml, ".yml"),
    (ConfigFormat::Yaml, ".yaml"),
    (ConfigFormat::Ron, ".ron"),
    (ConfigFormat::Toml, ".toml"),
];

fn parse_str_with_format<T: DeserializeOwned>(
    s: &str,
    config_format: ConfigFormat,
) -> Result<T, ConfigDeserializeErrorKind> {
    let res: T = match config_format {
        ConfigFormat::Json => serde_json::from_str(s)?,
        ConfigFormat::Yaml => serde_yaml::from_str(s)?,
        ConfigFormat::Ron => ron::from_str(s)?,
        ConfigFormat::Toml => toml::from_str(s)?,
        ConfigFormat::CargoMetadata => {
            return Err(ConfigDeserializeErrorKind::CargoMetadataError(
                CargoMetadataError::NoData,
            ));
        }
    };
    Ok(res)
}

pub enum ConfigFileContent {
    File(File),
}

#[non_exhaustive]
pub struct ConfigFileData<T> {
    pub filename: String,
    pub format: ConfigFormat,
    pub data: T,
}

pub fn find_config_file_in_cwd<T: DeserializeOwned>(
    name: &str,
) -> Result<Option<ConfigFileData<T>>, ConfigDeserializeError> {
    for (format, ext) in EXTENSIONS {
        let filename = format!("{}{}", name, ext);
        if let Ok(s) = fs::read_to_string(&filename) {
            match parse_str_with_format(&s, format) {
                Ok(data) => {
                    return Ok(Some(ConfigFileData {
                        filename,
                        format,
                        data,
                    }));
                }
                Err(kind) => {
                    return Err(ConfigDeserializeError {
                        filename,
                        format,
                        kind,
                    });
                }
            }
        }
    }

    let filename = "Cargo.toml";
    let format = ConfigFormat::CargoMetadata;
    if let Ok(s) = fs::read_to_string(filename) {
        let v = toml::Value::from_str(&s).or_else(|err| {
            Err(ConfigDeserializeError {
                filename: filename.to_string(),
                format,
                kind: ConfigDeserializeErrorKind::CargoMetadataError(
                    CargoMetadataError::InvalidToml(err),
                ),
            })
        })?;

        match v {
            toml::Value::Table(mut v) => {
                let pkg = v.remove(&format!("package.metadata.{}", name));
                let wsp = v.remove(&format!("workspace.metadata.{}", name));

                let v = if let Some(pkg) = pkg {
                    if let Some(_) = wsp {
                        return Err(ConfigDeserializeError {
                            filename: filename.to_string(),
                            format,
                            kind: ConfigDeserializeErrorKind::CargoMetadataError(
                                CargoMetadataError::FoundMultiple,
                            ),
                        });
                    } else {
                        pkg
                    }
                } else {
                    if let Some(v) = wsp {
                        v
                    } else {
                        return Ok(None);
                    }
                };

                let data: T = v.try_into().or_else(|err| {
                    Err(ConfigDeserializeError {
                        filename: filename.to_string(),
                        format,
                        kind: ConfigDeserializeErrorKind::CargoMetadataError(
                            CargoMetadataError::InvalidDataStructure(err),
                        ),
                    })
                })?;

                return Ok(Some(ConfigFileData {
                    filename: "Cargo.toml".to_string(),
                    format: ConfigFormat::CargoMetadata,
                    data,
                }));
            }
            _ => {
                return Err(ConfigDeserializeError {
                    filename: filename.to_string(),
                    format,
                    kind: ConfigDeserializeErrorKind::CargoMetadataError(
                        CargoMetadataError::CargoTomlIsNotTable,
                    ),
                })
            }
        }
    }

    Ok(None)
}
