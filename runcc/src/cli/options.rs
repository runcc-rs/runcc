use std::collections::HashMap;

use clap::{AppSettings, Parser};

use super::OptionsError;
use crate::{read, KillBehavior, RunConfig};

/// Run commands concurrently
#[derive(Parser)]
#[clap(version, author, bin_name = "cargo runcc")]
pub struct Opts {
    /// Commands to run concurrently
    command: Vec<String>,
    /// Config file path.
    ///
    /// Can't be used with positional arguments.
    /// See https://github.com/runcc-rs/runcc#usage for details
    #[clap(short, long)]
    config: Option<Option<String>>,
    /// Max length to print label in logs
    ///
    /// Defaults to the max length of all labels
    #[clap(long)]
    max_label_length: Option<usize>,
    /// Specify env vars with K=V
    #[clap(short, long)]
    env: Vec<String>,
    /// What to do after some command exits
    ///
    /// -k None (default)   : do nothing
    ///
    /// -k WhenAnyExited    : kill all commands when any exited
    ///
    /// -k WhenAnySucceeded : kill all commands when any exited with status == 0
    ///
    /// -k WhenAnyFailed    : kill all commands when any exited with status != 0
    ///
    /// -k <NUMBER>         : kill all commands when any exited with status == <NUMBER>
    #[clap(short, long)]
    kill: Option<KillBehavior>,
}

impl Opts {
    pub fn try_into_config(self) -> Result<RunConfig, OptionsError> {
        use crate::{CommandConfigInput, CommandConfigsInput, RunConfigInput};

        let Self {
            command: commands,
            config,
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

        // if no commands and no config are given, act as if -c was passed, i.e.
        // default to searching for the default runcc.* file in the local
        // directory
        let (commands, config) = match (&commands[..], &config) {
            ([], None) => (Vec::new(), Some(None)),
            _ => (commands, config),
        };

        if commands.len() > 0 {
            if config.is_some() {
                return Err(OptionsError::DuplicateConfigs);
            }

            Ok(RunConfigInput {
                commands: CommandConfigsInput::Commands(
                    commands
                        .into_iter()
                        .map(CommandConfigInput::Command)
                        .collect(),
                ),
                max_label_length,
                kill: kill.unwrap_or_default(),
                envs,
                windows_call_cmd_with_env: Default::default(),
            }
            .into())
        } else if let Some(config) = config {
            let data: read::ConfigFileData<RunConfigInput> =
                read::find_config_file(config.as_ref().and_then(|s| Some(s.as_str())), "runcc")
                    .or_else(|err| Err(OptionsError::ConfigFileError(err)))?;

            eprintln!("[runcc][info] using config file {:?}", data.filename);

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

            Ok(config)
        } else {
            Err(OptionsError::NoConfigs)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Opts;
    use clap::Parser;

    #[test]
    fn parse_multiple_env() {
        let opts = Opts::parse_from(["test", "--env", "A=a", "--env", "B=1"]);
        assert_eq!(opts.env, ["A=a", "B=1"]);
    }
}
