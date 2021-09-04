use serde::{Deserialize, Serialize};
use std::process::Command;

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

impl CommandConfig {
    pub fn from_script(script: &str) -> CommandConfig {
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
        if program.contains(" ") {
            if cfg!(target_os = "windows") {
                CommandConfig {
                    program: "cmd".to_string(),
                    args: Some(vec!["/C".to_string(), program.clone()]),
                    label: Some(program),
                    envs,
                    cwd: None,
                }
            } else {
                CommandConfig {
                    program: "sh".to_string(),
                    args: Some(vec!["-c".to_string(), program.clone()]),
                    label: Some(program),
                    envs,
                    cwd: None,
                }
            }
        } else {
            CommandConfig {
                program,
                args: None,
                label: None,
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

    pub fn into_command_and_label(
        self,
        inherited_envs: Option<Vec<(String, String)>>,
    ) -> (Command, String) {
        let Self {
            program,
            args,
            label,
            envs,
            cwd,
        } = self;

        let mut command = Command::new(&program);

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
}
