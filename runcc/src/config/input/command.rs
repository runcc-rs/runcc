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

impl CommandConfigInput {
    pub fn into_config(self, options: &CommandConfigFromScriptOptions) -> CommandConfig {
        match self {
            CommandConfigInput::Command(script) => CommandConfig::from_script(&script, options),
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

impl CommandConfigsInput {
    pub fn into_configs(self, options: &CommandConfigFromScriptOptions) -> Vec<CommandConfig> {
        match self {
            CommandConfigsInput::Commands(commands) => commands
                .into_iter()
                .map(|cmd| cmd.into_config(options))
                .collect(),
            CommandConfigsInput::LabeledCommands(map) => map
                .into_iter()
                .map(|(label, command)| match command {
                    Some(command) => {
                        let mut command: CommandConfig = command.into_config(options);
                        command.label = Some(label);

                        command
                    }
                    None => CommandConfig::from_program_args(label, None),
                })
                .collect(),
        }
    }
}
