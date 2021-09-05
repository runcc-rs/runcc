use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, fmt::Display};

pub struct InvalidEnvName(String);

impl Display for InvalidEnvName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid env name {}", self.0)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(try_from = "String")]
#[serde(into = "String")]
pub struct EnvName(String);

impl TryFrom<String> for EnvName {
    type Error = InvalidEnvName;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if EnvName::check_str(&value) {
            Ok(Self(value))
        } else {
            Err(InvalidEnvName(value))
        }
    }
}

impl Into<String> for EnvName {
    fn into(self) -> String {
        self.0
    }
}

impl EnvName {
    pub fn to_string(self) -> String {
        self.0
    }

    pub fn check_str(s: &str) -> bool {
        s.len() > 0
            && s.chars().enumerate().all(|(i, c)| match c {
                'a'..='z' | 'A'..='Z' => true,
                '0'..='9' | '_' => i > 0,
                _ => false,
            })
    }
}

#[non_exhaustive]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum WindowsCallCmdWithEnv {
    Random,
    EnvName(EnvName),
    Disable,
}

#[cfg(windows)]
fn get_random_env_name() -> String {
    use rand::{thread_rng, Rng};

    let mut rng = thread_rng();

    (0..8).map(|_| rng.gen_range('A'..='Z')).collect()
}

#[cfg(not(windows))]
fn get_random_env_name() -> String {
    "RANDNAME".to_string()
}

impl WindowsCallCmdWithEnv {
    pub fn try_into_env_name(self) -> Option<String> {
        match self {
            Self::Random => Some(format!("RUNCC_WIN_CMD__{}", get_random_env_name())),
            Self::EnvName(env_name) => Some(env_name.to_string()),
            Self::Disable => None,
        }
    }
}

impl Default for WindowsCallCmdWithEnv {
    fn default() -> Self {
        Self::Random
    }
}

impl Display for WindowsCallCmdWithEnv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

/// strictly validate env name [a-zA-Z][a-zA-Z0-9_]*
#[cfg(test)]
mod tests {
    use super::EnvName;

    #[test]
    fn test_is_valid_env_name() {
        assert!(EnvName::check_str("MY_ENV_0123_"));

        assert!(!EnvName::check_str(""));
        assert!(!EnvName::check_str(" "));
        assert!(!EnvName::check_str("123abc"));
        assert!(!EnvName::check_str("_MY_ENV"));
    }
}
