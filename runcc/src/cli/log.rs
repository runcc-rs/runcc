use std::fmt::Display;
use std::sync::{Arc, Mutex};
use tokio::io::AsyncBufReadExt;
use tokio::{io::BufReader, task::JoinHandle};

use crate::run::{kill, CommandStopped, CommandSystemPlugin, LabeledCommandData};

pub struct CommandSystemLogPlugin(Mutex<Vec<JoinHandle<()>>>);

impl CommandSystemLogPlugin {
    pub fn new() -> Self {
        Self(Default::default())
    }
}

impl CommandSystemPlugin<LabeledCommandData> for CommandSystemLogPlugin {
    type CommandInitialData = LabeledCommandData;

    fn initialize_command_data(
        &self,
        data: Self::CommandInitialData,
        stdout: tokio::process::ChildStdout,
        stderr: tokio::process::ChildStderr,
    ) -> LabeledCommandData {
        let label = data.label.display().to_string();

        let join = tokio::spawn(async move {
            tokio::join!(
                async {
                    let stdout = BufReader::new(stdout);
                    let mut lines = stdout.lines();
                    loop {
                        match lines.next_line().await {
                            Ok(line) => {
                                if let Some(line) = line {
                                    #[cfg(feature = "auto_ansi_escape")]
                                    let line = crate::ansi_escape::process_ansi_escape_line(
                                        label.len() + 3,
                                        &line,
                                    );

                                    let line = format!("[{}] {}", label, line);
                                    println!("{}", line);
                                } else {
                                    break;
                                }
                            }
                            Err(err) => {
                                eprintln!(
                                    "[runcc error] failed to read line from [{}] stdout: {}",
                                    label, err
                                );
                                break;
                            }
                        }
                    }
                },
                async {
                    let stderr = BufReader::new(stderr);
                    let mut lines = stderr.lines();
                    loop {
                        match lines.next_line().await {
                            Ok(line) => {
                                if let Some(line) = line {
                                    #[cfg(feature = "auto_ansi_escape")]
                                    let line = crate::ansi_escape::process_ansi_escape_line(
                                        label.len() + 3,
                                        &line,
                                    );

                                    let line = format!("[{}] {}", label, line);
                                    eprintln!("{}", line);
                                } else {
                                    break;
                                }
                            }
                            Err(err) => {
                                eprintln!(
                                    "[runcc error] failed to read line from [{}] stderr: {}",
                                    label, err
                                );
                                break;
                            }
                        }
                    }
                }
            );
        });

        let mut joins = self.0.lock().unwrap();
        joins.push(join);

        data
    }

    fn on_command_exited(&self, cmd: Arc<CommandStopped<LabeledCommandData, LabeledCommandData>>) {
        let label = cmd.data.label.display();
        let status = &cmd.exit_status;
        let killed = &cmd.killed;
        let status = match status {
            Ok(s) => format!(
                "code {}",
                s.code()
                    .map_or_else(|| "None".to_string(), |code| format!("{}", code))
            ),
            Err(err) => format!("error: {}", err),
        };

        let killed: std::borrow::Cow<str> = match killed {
            Some(kill_status) => {
                use crate::run::kill::KillJoinHandleFinalStatus as KS;
                match kill_status {
                    KS::Killed(reason) => format!(" (killed due to {})", reason).into(),
                    KS::FailedToKill { reason, error } => {
                        format!(" (tried to kill due to {} but failed: {})", reason, error).into()
                    }
                    _ => "".into(),
                }
            }
            None => "".into(),
        };

        let line = format!("[{}] exited with status {}{}", label, status, killed);
        eprintln!("{}", line);
    }

    fn join(&self) -> Option<tokio::task::JoinHandle<()>> {
        let mut joins = self.0.lock().unwrap();

        let mut joins: Vec<_> = joins.drain(0..).collect();

        Some(tokio::spawn(async move {
            for join in joins.iter_mut() {
                let _ = join.await;
            }
        }))
    }

    fn initialize_spawn_failed_command_data(
        &self,
        data: Self::CommandInitialData,
    ) -> LabeledCommandData {
        data
    }
}

impl Display for kill::KillCommandReason<LabeledCommandData> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            kill::KillCommandReason::OtherCommandExited(cmd) => {
                write!(f, "command[{}] exited", cmd.data.label.label())
            }
            kill::KillCommandReason::MainProcessGotSignal => write!(f, "Ctrl-C signal"),
        }
    }
}
