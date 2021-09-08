use std::{io, process::ExitStatus};

use super::super::kill;

#[non_exhaustive]
pub struct CommandStopped<T, R> {
    pub data: T,
    pub exit_status: io::Result<ExitStatus>,
    pub killed: Option<kill::KillJoinHandleFinalStatus<R>>,
}

impl<T, R> CommandStopped<T, R> {
    pub fn with_data<S>(self, new_data: S) -> (T, CommandStopped<S, R>) {
        let Self {
            data,
            exit_status,
            killed,
        } = self;
        (
            data,
            CommandStopped {
                data: new_data,
                exit_status,
                killed,
            },
        )
    }
}
