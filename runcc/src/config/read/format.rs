use serde::de::DeserializeOwned;
use std::{fs, fs::File, io, path::Path, str::FromStr};

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

pub fn find_config_file_in_dir<T: DeserializeOwned>(
    dir_path: &Path,
    app_name: &str,
) -> Result<ConfigFileData<T>, FindConfigError> {
    let mut patterns = Vec::with_capacity(EXTENSIONS.len() + 1);
    for (format, ext) in EXTENSIONS {
        let filename = format!("{}{}", app_name, ext);
        let file = dir_path.join(&filename);
        patterns.push(filename);

        match read_config_from_file_and_format(&file, format) {
            Ok(conf) => return Ok(conf),
            Err(err) => match err {
                ReadConfigError::OpenFileError { error, .. }
                    if error.kind() == io::ErrorKind::NotFound =>
                {
                    // file not found
                }
                err => return Err(FindConfigError::ReadError(err)),
            },
        };
    }

    match read_config_from_cargo_toml(&dir_path.join("Cargo.toml"), app_name) {
        Ok(Some(conf)) => return Ok(conf),
        Ok(None) => {
            // read Cargo.toml but no field
        }
        Err(err) => {
            match err {
                ReadConfigError::OpenFileError { error, .. }
                    if error.kind() == io::ErrorKind::NotFound =>
                {
                    // Cargo.toml not found
                }
                err => return Err(FindConfigError::ReadError(err)),
            }
        }
    };

    patterns.push("Cargo.toml".to_string());

    let dir = dir_path.to_string_lossy();

    let dir = if dir.is_empty() {
        "current working directory".to_string()
    } else {
        dir.into_owned()
    };

    Err(FindConfigError::NoFileMatch { patterns, dir })
}

pub fn read_config_from_file_and_format<T: DeserializeOwned>(
    file_path: &Path,
    format: ConfigFormat,
) -> Result<ConfigFileData<T>, ReadConfigError> {
    let filename = file_path.to_string_lossy().into_owned();

    match fs::read_to_string(file_path) {
        Ok(s) => match parse_str_with_format(&s, format) {
            Ok(data) => Ok(ConfigFileData {
                filename,
                format,
                data,
            }),
            Err(kind) => Err(ConfigDeserializeError {
                filename,
                format,
                kind,
            }
            .into()),
        },
        Err(error) => Err(ReadConfigError::OpenFileError {
            error,
            file: filename,
        }),
    }
}

pub fn read_config_from_cargo_toml<T: DeserializeOwned>(
    file_path: &Path,
    app_name: &str,
) -> Result<Option<ConfigFileData<T>>, ReadConfigError> {
    let filename = file_path.to_string_lossy().into_owned();
    let format = ConfigFormat::CargoMetadata;

    let s = match fs::read_to_string(file_path) {
        Ok(s) => s,
        Err(error) => {
            return Err(ReadConfigError::OpenFileError {
                file: filename,
                error,
            })
        }
    };

    let v = match toml::Value::from_str(&s) {
        Ok(v) => v,
        Err(err) => {
            return Err(ConfigDeserializeError {
                filename,
                format,
                kind: ConfigDeserializeErrorKind::CargoMetadataError(
                    CargoMetadataError::InvalidToml(err),
                ),
            }
            .into());
        }
    };

    match v {
        toml::Value::Table(mut v) => {
            let pkg = v.remove(&format!("package.metadata.{}", app_name));
            let wsp = v.remove(&format!("workspace.metadata.{}", app_name));

            let v = if let Some(pkg) = pkg {
                if let Some(_) = wsp {
                    return Err(ConfigDeserializeError {
                        filename: filename.to_string(),
                        format,
                        kind: ConfigDeserializeErrorKind::CargoMetadataError(
                            CargoMetadataError::FoundMultiple,
                        ),
                    }
                    .into());
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
            }
            .into())
        }
    }
}

pub fn find_config_file<T: DeserializeOwned>(
    file_or_dir_path: Option<&str>,
    app_name: &str,
) -> Result<ConfigFileData<T>, FindConfigError> {
    let file_or_dir_path = file_or_dir_path.unwrap_or("");

    let path = Path::new(file_or_dir_path);

    if file_or_dir_path.is_empty() || file_or_dir_path == "." || path.is_dir() {
        find_config_file_in_dir(path, app_name)
    } else {
        let filename = path.file_name().unwrap_or_default();
        if filename == "Cargo.toml" {
            if let Some(conf) = read_config_from_cargo_toml(path, app_name)? {
                Ok(conf)
            } else {
                // Cargo.toml exists but lack config
                Err(FindConfigError::ReadError(
                    ReadConfigError::DeserializeError(ConfigDeserializeError {
                        filename: path.to_string_lossy().into_owned(),
                        format: ConfigFormat::CargoMetadata,
                        kind: ConfigDeserializeErrorKind::CargoMetadataError(
                            CargoMetadataError::NoData,
                        ),
                    }),
                ))
            }
        } else {
            let filename = filename.to_string_lossy();
            for (format, ext) in EXTENSIONS {
                if filename.ends_with(ext) {
                    let conf = read_config_from_file_and_format(path, format)?;

                    return Ok(conf);
                }
            }

            Err(FindConfigError::UnknownExtension {
                extension: path
                    .extension()
                    .map(|ext| ext.to_string_lossy().into_owned()),
                file: path.to_string_lossy().into_owned(),
            })
        }
    }
}
