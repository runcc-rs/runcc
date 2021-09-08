use std::{io, process::Stdio};

use tokio::process::{Child, ChildStderr, ChildStdout, Command};
use tokio::sync::oneshot;

use super::super::kill;

pub struct CommandInitialized<T> {
    command: Command,
    data: T,
}

fn start_kill_child_process<T>(
    child: &mut Child,
    kill_reason: kill::KillCommandReason<T>,
) -> kill::KillJoinHandleFinalStatus<T> {
    if let Err(kill_err) = child.start_kill() {
        match kill_err.kind() {
            io::ErrorKind::InvalidInput | io::ErrorKind::PermissionDenied => {
                // child process has exited
                kill::KillJoinHandleFinalStatus::AlreadyExited(
                    kill::CommandAlreadyExitedKind::ProcessExited(kill_reason),
                )
            }
            _ => kill::KillJoinHandleFinalStatus::FailedToKill {
                error: kill_err,
                reason: kill_reason,
            },
        }
    } else {
        kill::KillJoinHandleFinalStatus::Killed(kill_reason)
    }
}

impl<T> CommandInitialized<T> {
    pub fn new(command: Command, data: T) -> Self {
        Self { command, data }
    }

    pub fn spawn<R: 'static + std::marker::Sync + std::marker::Send>(
        self,
    ) -> io::Result<(super::CommandSpawned<T, R>, ChildStdout, ChildStderr)> {
        let Self { mut command, data } = self;
        let (kill_sender, kill_receiver) = oneshot::channel::<kill::KillCommandReason<R>>();

        let mut child = command
            .env("CARGO_TERM_COLOR", "always")
            // yarn force color https://classic.yarnpkg.com/en/docs/cli/#toc-verbose
            .env("FORCE_COLOR", "true")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let stdout = child.stdout.take().ok_or_else(|| {
            io::Error::new(io::ErrorKind::Other, "Could not capture standard output.")
        })?;
        let stderr = child.stderr.take().ok_or_else(|| {
            io::Error::new(io::ErrorKind::Other, "Could not capture standard error.")
        })?;

        let join_handle = tokio::spawn(async move {
            tokio::select! {
                status = child.wait() => (status, None),
                kill_reason = kill_receiver => {
                    let kill_status = if let Ok(kill_reason) = kill_reason {
                        start_kill_child_process(&mut child, kill_reason)
                    } else {
                        kill::KillJoinHandleFinalStatus::SenderDisconnected
                    };
                    let status = child.wait().await;
                    (status, Some(kill_status))
                }
            }
        });

        Ok((
            super::CommandSpawned::new(data, kill_sender, join_handle),
            stdout,
            stderr,
        ))
    }
}
