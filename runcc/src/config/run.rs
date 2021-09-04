use serde::{Deserialize, Serialize};

use super::CommandConfig;

#[non_exhaustive]
#[derive(Deserialize, Serialize, Debug)]
pub struct RunConfig {
    pub commands: Vec<CommandConfig>,
    pub max_label_length: usize,
}
