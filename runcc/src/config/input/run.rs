use serde::{Deserialize, Serialize};
use std::cmp;

use super::super::{run::*, CommandConfig};
use super::CommandConfigsInput;

#[non_exhaustive]
#[derive(Deserialize, Serialize)]
pub struct RunConfigInput {
    pub commands: CommandConfigsInput,
    pub max_label_length: Option<usize>,
}

impl Into<RunConfig> for RunConfigInput {
    fn into(self) -> RunConfig {
        let Self {
            commands,
            max_label_length,
        } = self;

        let commands: Vec<CommandConfig> = commands.into();

        let real_max_label_length = commands
            .iter()
            .map(|cmd| cmd.label_length())
            .max()
            .unwrap_or(0);

        let max_label_length = match max_label_length {
            Some(0) | None => real_max_label_length,
            Some(v) => cmp::min(v, real_max_label_length),
        };

        RunConfig {
            commands,
            max_label_length,
        }
    }
}
