use serde::{Deserialize, Serialize};

use super::super::{ExitStatusPattern, KillBehavior};

#[derive(Deserialize, Serialize, Debug)]
#[non_exhaustive]
pub enum KillBehaviorInputStr {
    None,
    WhenAnyExited,
    WhenAnySucceeded,
    WhenAnyFailed,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
#[non_exhaustive]
pub enum KillBehaviorInput {
    Str(KillBehaviorInputStr),
    WhenAnyExitedWithStatus(i32),
}

impl From<KillBehaviorInput> for KillBehavior {
    fn from(val: KillBehaviorInput) -> Self {
        match val {
            KillBehaviorInput::Str(val) => match val {
                KillBehaviorInputStr::None => KillBehavior::None,
                KillBehaviorInputStr::WhenAnyExited => KillBehavior::WhenAnyExited,
                KillBehaviorInputStr::WhenAnySucceeded => {
                    KillBehavior::WhenAnyExitedWithStatus(ExitStatusPattern::Success)
                }
                KillBehaviorInputStr::WhenAnyFailed => {
                    KillBehavior::WhenAnyExitedWithStatus(ExitStatusPattern::Failed)
                }
            },
            KillBehaviorInput::WhenAnyExitedWithStatus(s) => {
                KillBehavior::WhenAnyExitedWithStatus(ExitStatusPattern::StatusCode(s))
            }
        }
    }
}
