use serde::{Deserialize, Serialize};

use crate::env::match_program_with_envs;

#[non_exhaustive]
#[derive(Deserialize, Serialize, Debug)]
pub struct CommandConfig {
    pub program: String,
    pub args: Option<Vec<String>>,
    pub label: Option<String>,
    pub envs: Option<Vec<(String, String)>>,
    pub cwd: Option<String>,
}

#[non_exhaustive]
#[derive(Debug, Default)]
pub struct CommandConfigFromScriptOptions {
    pub windows_call_cmd_with_env: super::WindowsCallCmdWithEnv,
}

macro_rules! def_into_command_and_label {
    ($name:ident -> $cmd_type:ty) => {
        pub fn $name<I, K, V>(self, inherited_envs: Option<I>) -> ($cmd_type, String)
        where
            I: IntoIterator<Item = (K, V)>,
            K: AsRef<std::ffi::OsStr>,
            V: AsRef<std::ffi::OsStr>,
        {
            let Self {
                program,
                args,
                label,
                envs,
                cwd,
            } = self;

            let mut command = <$cmd_type>::new(&program);

            if let Some(cwd) = cwd {
                command.current_dir(cwd);
            }

            if let Some(args) = &args {
                command.args(args);
            }

            if let Some(envs) = inherited_envs {
                command.envs(envs);
            }

            if let Some(envs) = envs {
                command.envs(envs);
            }

            let label = label.unwrap_or_else(move || {
                if let Some(args) = args {
                    format!("{} {}", program, args.join(" "))
                } else {
                    program
                }
            });

            (command, label)
        }
    };
}

impl CommandConfig {
    pub fn from_script(script: &str, options: &CommandConfigFromScriptOptions) -> CommandConfig {
        let script = script.trim();

        let (program, envs) = match_program_with_envs(script);

        let program = program.to_string();
        let envs = if let Some(envs) = envs {
            Some(
                envs.into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
            )
        } else {
            None
        };

        if cfg!(target_os = "windows") {
            let with_env = &options.windows_call_cmd_with_env;

            let env_name = with_env.clone().try_into_env_name();

            let (arg, env) = match env_name {
                Some(env_name) => (format!("%{}%", env_name), Some((env_name, program.clone()))),
                None => (program.clone(), None),
            };

            let mut cmd = CommandConfig {
                program: "cmd".to_string(),
                args: Some(vec!["/C".to_string(), arg]),
                label: Some(program.clone()),
                envs,
                cwd: None,
            };

            if let Some(env) = env {
                cmd.env(env);
            }

            cmd
        } else {
            CommandConfig {
                program: "sh".to_string(),
                args: Some(vec!["-c".to_string(), program.clone()]),
                label: Some(program),
                envs,
                cwd: None,
            }
        }
    }

    pub fn from_program_args(program: String, args: Option<Vec<String>>) -> CommandConfig {
        CommandConfig {
            program,
            args,
            label: None,
            envs: None,
            cwd: None,
        }
    }

    pub fn label_length(&self) -> usize {
        match &self.label {
            None => {
                self.program.len()
                    + self
                        .args
                        .as_ref()
                        .map_or(0, |args| args.iter().map(|arg| arg.len() + 1).sum())
            }
            Some(label) => label.len(),
        }
    }

    pub fn env(&mut self, env: (String, String)) -> &mut Self {
        self.envs.get_or_insert_with(|| vec![]).push(env);
        self
    }

    def_into_command_and_label! {into_command_and_label->std::process::Command}

    def_into_command_and_label! {into_tokio_command_and_label->tokio::process::Command}
}
