use std::{
    io,
    sync::{Arc, Mutex},
};

use tokio::sync::oneshot;

use super::command::CommandStopped;

pub enum KillCommandReason<T> {
    OtherCommandExited(Arc<CommandStopped<T, T>>),
    MainProcessGotSignal,
}

impl<T> Clone for KillCommandReason<T> {
    fn clone(&self) -> Self {
        match self {
            Self::OtherCommandExited(arc) => Self::OtherCommandExited(arc.clone()),
            Self::MainProcessGotSignal => Self::MainProcessGotSignal,
        }
    }
}

pub(super) type KillSender<T> = oneshot::Sender<KillCommandReason<T>>;
// pub(super) type KillReceiver<T> = oneshot::Receiver<KillCommandReason<T>>;

#[derive(Clone)]
pub struct CommandKiller<T>(Arc<Mutex<Option<KillSender<T>>>>);

impl<T> CommandKiller<T> {
    pub(super) fn new(kill_sender: KillSender<T>) -> Self {
        Self(Arc::new(Mutex::new(Some(kill_sender))))
    }

    pub fn kill(&self, reason: KillCommandReason<T>) -> KillResult {
        let mut kill_sender = self.0.lock().unwrap();

        let kill_sender = kill_sender.take();
        if let Some(kill_sender) = kill_sender {
            match kill_sender.send(reason) {
                Ok(_) => KillResult::SentSuccess,
                Err(_) => KillResult::AlreadyExited,
            }
        } else {
            KillResult::AlreadySent
        }
    }
}
// pub type KillSender = mpsc::Sender<KillSenderValue>;

// pub enum KillSenderValue {
//     Kill(KillCommandReason),
//     SelfExited,
// }

pub enum KillJoinHandleFinalStatus<T> {
    SenderDisconnected,
    Killed(KillCommandReason<T>),
    FailedToKill {
        reason: KillCommandReason<T>,
        error: io::Error,
    },
    AlreadyExited(CommandAlreadyExitedKind<T>),
    UnexpectedAlreadyKilled,
}

pub enum CommandAlreadyExitedKind<T> {
    SelfExited,
    ProcessExited(KillCommandReason<T>),
}

pub enum KillResult {
    SentSuccess,
    AlreadySent,
    AlreadyExited,
}
