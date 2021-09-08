use serde::{Deserialize, Serialize};
use std::fmt::Display;

use super::input::KillBehaviorInput;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub enum ExitStatusPattern {
    Success,
    Failed,
    StatusCode(i32),
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(from = "KillBehaviorInput")]
pub enum KillBehavior {
    None,
    WhenAnyExited,
    WhenAnyExitedWithStatus(ExitStatusPattern),
    // WhenLabeledExited,
}

impl Default for KillBehavior {
    fn default() -> Self {
        Self::None
    }
}

impl Display for KillBehavior {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KillBehavior::None => write!(f, "none"),
            KillBehavior::WhenAnyExited => write!(f, "kill other commands when any exited"),
            KillBehavior::WhenAnyExitedWithStatus(s) => {
                let s: std::borrow::Cow<str> = match s {
                    ExitStatusPattern::Success => "successfully".into(),
                    ExitStatusPattern::Failed => "with failure".into(),
                    ExitStatusPattern::StatusCode(code) => {
                        format!("with status code {}", code).into()
                    }
                };
                write!(f, "kill other commands when any exited {}", s)
            }
        }
    }
}

impl std::str::FromStr for KillBehavior {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let kill: KillBehaviorInput = serde_json::from_str(s)?;

        Ok(kill.into())
    }
}
