use std::io;
use std::process::ExitStatus;
use tokio::task::JoinHandle;

use super::super::kill;

#[derive(Clone)]
pub struct SharedCommandSpawned<T, R> {
    data: T,
    killer: kill::CommandKiller<R>,
}

pub(super) type CommandTokioJoinHandle<T> = JoinHandle<(
    io::Result<ExitStatus>,
    Option<kill::KillJoinHandleFinalStatus<T>>,
)>;

pub struct CommandJoinHandle<R>(CommandTokioJoinHandle<R>);

impl<R> CommandJoinHandle<R> {
    pub async fn join(self) -> super::CommandStopped<(), R> {
        let (exit_status, killed) = self.0.await.expect("command task handle should not panic");

        super::CommandStopped {
            data: (),
            exit_status,
            killed,
        }
    }
}

pub struct CommandSpawned<T, R> {
    pub data: T,
    pub killer: kill::CommandKiller<R>,
    pub join_handle: CommandJoinHandle<R>,
}

impl<T, R> CommandSpawned<T, R> {
    pub(super) fn new(
        data: T,
        kill_sender: kill::KillSender<R>,
        join_handle: CommandTokioJoinHandle<R>,
    ) -> Self {
        Self {
            data,
            killer: kill::CommandKiller::new(kill_sender),
            join_handle: CommandJoinHandle(join_handle),
        }
    }

    pub fn kill(&self, reason: kill::KillCommandReason<R>) -> kill::KillResult {
        self.killer.kill(reason)
    }

    pub async fn wait_into_stopped(self) -> super::CommandStopped<T, R> {
        let Self {
            data, join_handle, ..
        } = self;

        join_handle.join().await.with_data(data).1
    }

    pub fn with_data<S>(self, new_data: S) -> (T, CommandSpawned<S, R>) {
        let Self {
            data,
            join_handle,
            killer,
        } = self;

        (
            data,
            CommandSpawned {
                data: new_data,
                join_handle,
                killer,
            },
        )
    }
}

impl<T: Clone, R> CommandSpawned<T, R> {
    pub fn share_data(&self) -> T {
        self.data.clone()
    }
}

impl<T, R: Clone> CommandSpawned<T, R> {
    pub fn share_killer(&self) -> kill::CommandKiller<R> {
        self.killer.clone()
    }
}

impl<T: Clone, R: Clone> CommandSpawned<T, R> {
    pub fn share(&self) -> SharedCommandSpawned<T, R> {
        SharedCommandSpawned {
            data: self.data.clone(),
            killer: self.killer.clone(),
        }
    }
}
