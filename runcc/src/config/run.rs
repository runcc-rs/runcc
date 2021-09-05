use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::CommandConfig;

#[non_exhaustive]
#[derive(Deserialize, Serialize, Debug)]
pub struct RunConfig {
    pub commands: Vec<CommandConfig>,
    pub max_label_length: usize,
    pub envs: Option<HashMap<String, String>>,
}
