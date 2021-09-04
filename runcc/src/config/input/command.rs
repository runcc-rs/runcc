use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::super::command::*;

#[non_exhaustive]
#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum CommandConfigInput {
    Command(String),
    ProgramAndArgs(Vec<String>),
    CommandConfig(CommandConfig),
}

impl Into<CommandConfig> for CommandConfigInput {
    fn into(self) -> CommandConfig {
        match self {
            CommandConfigInput::Command(script) => CommandConfig::from_script(&script),
            CommandConfigInput::ProgramAndArgs(mut names) => {
                let program = if names.is_empty() {
                    String::new()
                } else {
                    names.remove(0)
                };
                CommandConfig::from_program_args(
                    program,
                    if names.is_empty() { None } else { Some(names) },
                )
            }
            CommandConfigInput::CommandConfig(config) => config,
        }
    }
}

#[non_exhaustive]
#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum CommandConfigsInput {
    Commands(Vec<CommandConfigInput>),
    LabeledCommands(HashMap<String, Option<CommandConfigInput>>),
}

impl Into<Vec<CommandConfig>> for CommandConfigsInput {
    fn into(self) -> Vec<CommandConfig> {
        match self {
            CommandConfigsInput::Commands(commands) => {
                commands.into_iter().map(Into::into).collect()
            }
            CommandConfigsInput::LabeledCommands(map) => map
                .into_iter()
                .map(|(label, command)| match command {
                    Some(command) => {
                        let mut command: CommandConfig = command.into();
                        command.label = Some(label);

                        command
                    }
                    None => CommandConfig::from_program_args(label, None),
                })
                .collect(),
        }
    }
}
