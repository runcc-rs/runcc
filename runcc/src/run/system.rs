use std::{
    cmp, mem,
    sync::{Arc, Mutex},
};

use tokio::{
    process::{ChildStderr, ChildStdout, Command},
    sync::{mpsc, Mutex as AsyncMutex},
    task::JoinHandle,
};

use crate::{label::Label, KillBehavior, RunConfig};

use super::kill;
use super::{
    command::{CommandInitialized, CommandSpawned, CommandStopped},
    CommandSystemSimpleReport,
};

enum CommandState<T> {
    Processing,
    Spawned {
        data: T,
        killer: kill::CommandKiller<T>,
    },
    Stopped(Arc<CommandStopped<T, T>>),
}

#[derive(Clone)]
pub struct CommandSystemKiller<T>(mpsc::Sender<Option<Arc<CommandStopped<T, T>>>>);

impl<T> CommandSystemKiller<T> {
    pub async fn kill_all(&self) {
        let _ = self.0.send(None).await;
    }
}

pub struct CommandSystem<T, P>
where
    P: CommandSystemPlugin<T>,
{
    commands: Arc<Vec<Arc<Mutex<CommandState<T>>>>>,
    killer: CommandSystemKiller<T>,
    handles: AsyncMutex<Option<CommandSystemHandles>>,
    plugin: Arc<P>,
}

struct CommandSystemHandles {
    commands_handles: Vec<JoinHandle<()>>,
    killer_handle: JoinHandle<()>,
}

impl<T, P> CommandSystem<T, P>
where
    T: std::marker::Send + std::marker::Sync + 'static,
    P: CommandSystemPlugin<T>,
{
    fn spawn_with_plugin<I>(commands: I, kill_behavior: KillBehavior, plugin: P) -> Self
    where
        I: IntoIterator<Item = (Command, P::CommandInitialData)>,
        P: ,
    {
        let commands: Vec<_> = commands.into_iter().collect();
        let (tx, mut rx) = mpsc::channel(cmp::min(commands.len(), 1));

        let plugin = Arc::new(plugin);

        let (commands, handles): (Vec<_>, Vec<_>) = commands
            .into_iter()
            .map(|(command, data)| {
                let tx = tx.clone();
                let plugin = plugin.clone();

                let spawned = CommandInitialized::new(command, ()).spawn::<T>();

                let (cmd, stdout, stderr) = match spawned {
                    Ok((cmd, stdout, stderr)) => (cmd.with_data(data).1, stdout, stderr),
                    Err(err) => {
                        let data = plugin.initialize_spawn_failed_command_data(data);
                        let cmd = Arc::new(CommandStopped {
                            data,
                            exit_status: Err(err),
                            killed: None,
                        });
                        let mutex = Arc::new(Mutex::new(CommandState::Stopped(cmd.clone())));

                        let handle = tokio::spawn(async move {
                            &plugin.on_command_exited(cmd.clone());
                            let _ = tx.send(Some(cmd)).await;
                        });

                        return (mutex, handle);
                    }
                };

                let CommandSpawned {
                    data,
                    join_handle,
                    killer,
                } = cmd;

                let data = plugin.initialize_command_data(data, stdout, stderr);

                let mutex_ret = Arc::new(Mutex::new(CommandState::Spawned { data, killer }));

                let mutex = mutex_ret.clone();

                let handle = tokio::spawn(async move {
                    let cmd = join_handle.join().await;

                    let cmd = {
                        let mut state = mutex.lock().unwrap();

                        let old_state = mem::replace(&mut *state, CommandState::Processing);

                        let cmd = match old_state {
                            CommandState::Spawned { data, killer: _ } => {
                                let cmd = Arc::new(cmd.with_data(data).1);

                                *state = CommandState::Stopped(cmd.clone());

                                if tx.is_closed() {
                                    &plugin.on_command_exited(cmd);
                                    None
                                } else {
                                    &plugin.on_command_exited(cmd.clone());
                                    Some(cmd)
                                }
                            }
                            _ => panic!("unreachable"),
                        };

                        drop(state);
                        cmd
                    };

                    if let Some(cmd) = cmd {
                        let _ = tx.send(Some(cmd)).await;
                    }
                });

                (mutex_ret, handle)
            })
            .collect::<Vec<_>>()
            .into_iter()
            .unzip();

        let command_count = commands.len();
        let commands_ret = Arc::new(commands);

        let commands = commands_ret.clone();
        let killer_handle = tokio::spawn(async move {
            let mut exited_command_count = 0;
            while let Some(exited_cmd) = rx.recv().await {
                let reason = if let Some(ref exited_cmd) = exited_cmd {
                    exited_command_count += 1;

                    if exited_command_count >= command_count {
                        break;
                    }

                    let should_kill_all: bool = match &kill_behavior {
                        KillBehavior::None => false,
                        KillBehavior::WhenAnyExited => true,
                        KillBehavior::WhenAnyExitedWithStatus(status) => match status {
                            crate::ExitStatusPattern::Success => exited_cmd
                                .exit_status
                                .as_ref()
                                .ok()
                                .map_or(false, |s| s.success()),
                            crate::ExitStatusPattern::Failed => exited_cmd
                                .exit_status
                                .as_ref()
                                .ok()
                                .map_or(true, |s| !s.success()),
                            crate::ExitStatusPattern::StatusCode(code) => exited_cmd
                                .exit_status
                                .as_ref()
                                .ok()
                                .map_or(false, |s| s.code() == Some(*code)),
                        },
                    };

                    if should_kill_all {
                        Some(kill::KillCommandReason::OtherCommandExited(
                            exited_cmd.clone(),
                        ))
                    } else {
                        None
                    }
                } else {
                    // got kill all
                    Some(kill::KillCommandReason::MainProcessGotSignal)
                };

                if let Some(reason) = reason {
                    drop(rx);

                    for state in commands.iter() {
                        let mut state = state.lock().unwrap();

                        match &mut *state {
                            CommandState::Spawned { killer, .. } => {
                                killer.kill(reason.clone());
                            }
                            _ => {}
                        }
                    }

                    break;
                }
            }
        });

        Self {
            commands: commands_ret,
            killer: CommandSystemKiller(tx),
            handles: AsyncMutex::new(Some(CommandSystemHandles {
                commands_handles: handles,
                killer_handle,
            })),
            plugin,
        }
    }
}

impl<T: Clone, P: CommandSystemPlugin<T>> CommandSystem<T, P> {
    pub fn share_killer(&self) -> CommandSystemKiller<T> {
        self.killer.clone()
    }
}

impl<T, P: CommandSystemPlugin<T>> CommandSystem<T, P> {
    pub async fn kill_all(&self) {
        self.killer.kill_all().await;
    }

    async fn wait_iter_stopped_commands<'a, R, F>(
        &'a mut self,
        cmd_processor: F,
    ) -> impl Iterator<Item = R> + 'a
    where
        F: 'a + Fn(&Arc<CommandStopped<T, T>>) -> R,
    {
        let Self {
            commands,
            handles,
            plugin,
            ..
        } = self;

        let mut handles = handles.lock().await;

        if let Some(handles) = handles.take() {
            let CommandSystemHandles {
                commands_handles,
                killer_handle,
            } = handles;

            for handle in commands_handles {
                handle.await.expect("CommandSystem subtask panicked");
            }

            killer_handle
                .await
                .expect("CommandSystem's subtask for killing commands panicked");
        }

        drop(handles);

        if let Some(plugin_join) = plugin.join() {
            let _ = plugin_join.await;
        }

        commands.iter().map(move |cmd| {
            let cmd = cmd.lock().unwrap();

            match &*cmd {
                CommandState::Stopped(cmd) => cmd_processor(cmd),
                _ => panic!("CommandState should be stopped after handles joined"),
            }
        })
    }

    pub async fn wait(&mut self) -> CommandSystemSimpleReport {
        let command_count_total = self.commands.len();
        let mut command_count_success = 0usize;

        for success in self
            .wait_iter_stopped_commands(|cmd| {
                if let Ok(status) = cmd.exit_status {
                    status.success()
                } else {
                    false
                }
            })
            .await
        {
            if success {
                command_count_success += 1;
            }
        }

        CommandSystemSimpleReport {
            command_count_total,
            command_count_success,
        }
    }

    pub async fn wait_into_stopped_commands(&mut self) -> Vec<Arc<CommandStopped<T, T>>> {
        let commands = self.wait_iter_stopped_commands(Arc::clone).await.collect();

        commands
    }
}

#[derive(Debug, Clone)]
pub struct LabeledCommandData {
    pub label: Label,
}

pub fn spawn_from_run_config_with_plugin<T, P>(
    run_config: RunConfig,
    plugin: P,
) -> CommandSystem<T, P>
where
    T: Send + Sync + 'static,
    P: CommandSystemPlugin<T, CommandInitialData = LabeledCommandData>,
{
    let RunConfig {
        commands,
        max_label_length,
        envs,
        kill,
    } = run_config;

    let commands = commands.into_iter().map(|cmd| {
        let (cmd, label) = cmd.into_tokio_command_and_label(envs.as_ref());

        (
            cmd,
            LabeledCommandData {
                label: Label::from_label(label, max_label_length),
            },
        )
    });

    CommandSystem::spawn_with_plugin(commands, kill, plugin)
}

pub trait CommandSystemPlugin<T>: Send + Sync + 'static + Sized {
    type CommandInitialData;

    fn initialize_spawn_failed_command_data(&self, data: Self::CommandInitialData) -> T;

    fn initialize_command_data(
        &self,
        data: Self::CommandInitialData,
        stdout: ChildStdout,
        stderr: ChildStderr,
    ) -> T;

    fn on_command_exited(&self, _cmd: Arc<CommandStopped<T, T>>) {}

    fn join(&self) -> Option<JoinHandle<()>> {
        None
    }
}
