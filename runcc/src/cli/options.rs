use std::collections::HashMap;

use clap::{crate_authors, crate_version, AppSettings, Clap};

use super::OptionsError;
use crate::{read, RunConfig};

/// This doc string acts as a help message when the user runs '--help'
/// as do all doc strings on fields
#[derive(Clap)]
#[clap(version = crate_version!(), author = crate_authors!("\n"))]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Opts {
    command: Vec<String>,
    #[clap(long)]
    max_label_length: Option<usize>,
    #[clap(short, long)]
    env: Vec<String>,
}

impl Opts {
    pub fn try_into_config(self) -> Result<Option<RunConfig>, OptionsError> {
        use crate::{CommandConfigInput, CommandConfigsInput, RunConfigInput};

        let Self {
            command,
            max_label_length,
            env,
        } = self;

        if command.len() > 0 {
            Ok(Some(
                RunConfigInput {
                    commands: CommandConfigsInput::Commands(
                        command
                            .into_iter()
                            .map(CommandConfigInput::Command)
                            .collect(),
                    ),
                    max_label_length,
                    envs: if env.len() > 0 {
                        let envs: HashMap<String, String> = env
                            .into_iter()
                            .map(|env| {
                                let (kv, program) = crate::env::match_one_env(&env);

                                if let Some(kv) = kv {
                                    if program == "" {
                                        Ok((kv.0.to_string(), kv.1.to_string()))
                                    } else {
                                        Err(env)
                                    }
                                } else {
                                    Err(env)
                                }
                            })
                            .collect::<Result<_, _>>()
                            .or_else(|invalid_env| {
                                Err(OptionsError::EnvSyntaxError(invalid_env))
                            })?;

                        Some(envs)
                    } else {
                        None
                    },
                    windows_call_cmd_with_env: Default::default(),
                }
                .into(),
            ))
        } else {
            let data: Option<read::ConfigFileData<RunConfigInput>> =
                read::find_config_file_in_cwd("runcc")
                    .or_else(|err| Err(OptionsError::ConfigFileError(err)))?;

            Ok(data.and_then(|data| Some(data.data.into())))
        }
    }
}
