use std::collections::HashMap;

use clap::{crate_authors, crate_version, AppSettings, Clap};

use super::OptionsError;
use crate::{read, KillBehavior, RunConfig};

/// Run commands concurrently
#[derive(Clap)]
#[clap(version = crate_version!(), author = crate_authors!(", "), bin_name = "cargo runcc")]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Opts {
    command: Vec<String>,
    #[clap(long)]
    max_label_length: Option<usize>,
    #[clap(short, long)]
    env: Vec<String>,
    #[clap(short, long)]
    kill: Option<KillBehavior>,
}

impl Opts {
    pub fn try_into_config(self) -> Result<Option<RunConfig>, OptionsError> {
        use crate::{CommandConfigInput, CommandConfigsInput, RunConfigInput};

        let Self {
            command,
            max_label_length,
            env,
            kill,
        } = self;

        let envs = if env.len() > 0 {
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
                .or_else(|invalid_env| Err(OptionsError::EnvSyntaxError(invalid_env)))?;

            Some(envs)
        } else {
            None
        };

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
                    kill: kill.unwrap_or_default(),
                    envs,
                    windows_call_cmd_with_env: Default::default(),
                }
                .into(),
            ))
        } else {
            let data: Option<read::ConfigFileData<RunConfigInput>> =
                read::find_config_file_in_cwd("runcc")
                    .or_else(|err| Err(OptionsError::ConfigFileError(err)))?;

            if let Some(data) = data {
                let mut config: RunConfig = data.data.into();

                if let Some(envs) = envs {
                    eprintln!("[runcc][warning] env vars from cli args will be appended to envs from config file");
                    if let Some(old_envs) = &mut config.envs {
                        old_envs.extend(envs);
                    } else {
                        config.envs = Some(envs);
                    };
                }

                if let Some(max_label_length) = max_label_length {
                    if max_label_length != config.max_label_length {
                        eprintln!("[runcc][warning] max_label_length from cli args will override the value from config file");
                        config.max_label_length = max_label_length;
                    }
                }

                if let Some(kill) = kill {
                    if kill != config.kill {
                        eprintln!("[runcc][warning] kill from cli args will override the value from config file");
                        config.kill = kill;
                    }
                }

                Ok(Some(config))
            } else {
                Ok(None)
            }
        }
    }
}
