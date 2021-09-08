use serde::{Deserialize, Serialize};

use super::super::{ExitStatusPattern, KillBehavior};

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
#[non_exhaustive]
pub enum KillBehaviorInput {
    None,
    WhenAnyExited,
    WhenAnySucceeded,
    WhenAnyFailed,
    WhenAnyExitedWithStatus(i32),
}

impl From<KillBehaviorInput> for KillBehavior {
    fn from(val: KillBehaviorInput) -> Self {
        match val {
            KillBehaviorInput::None => KillBehavior::None,
            KillBehaviorInput::WhenAnyExited => KillBehavior::WhenAnyExited,
            KillBehaviorInput::WhenAnySucceeded => {
                KillBehavior::WhenAnyExitedWithStatus(ExitStatusPattern::Success)
            }
            KillBehaviorInput::WhenAnyFailed => {
                KillBehavior::WhenAnyExitedWithStatus(ExitStatusPattern::Failed)
            }
            KillBehaviorInput::WhenAnyExitedWithStatus(s) => {
                KillBehavior::WhenAnyExitedWithStatus(ExitStatusPattern::StatusCode(s))
            }
        }
    }
}
