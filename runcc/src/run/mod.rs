use ctrlc;
use std::fmt::Display;
use std::io::{BufRead, BufReader, Error, ErrorKind};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};

use crate::config::RunConfig;
use crate::label::display_label;

fn io_error_from<T: Display, R>(err: T) -> Result<R, Error> {
    Err(Error::new(ErrorKind::Other, format!("{}", err)))
}

fn command_stdout(
    mut cmd: Command,
    label: &str,
    rx_kill: mpsc::Receiver<bool>,
) -> Result<(ExitStatus, JoinHandle<Result<bool, Error>>), Error> {
    let mut child = cmd
        .env("CARGO_TERM_COLOR", "always")
        // yarn force color https://classic.yarnpkg.com/en/docs/cli/#toc-verbose
        .env("FORCE_COLOR", "true")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| Error::new(ErrorKind::Other, "Could not capture standard output."))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| Error::new(ErrorKind::Other, "Could not capture standard error."))?;

    let join_out = {
        let label = label.to_string();
        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            reader
                .lines()
                .filter_map(|line| line.ok())
                .for_each(|line| {
                    let line = format!("[{}] {}", label, line);
                    println!("{}", line);
                });
        })
    };

    let join_err = {
        let label = label.to_string();
        thread::spawn(move || {
            let reader_err = BufReader::new(stderr);
            reader_err
                .lines()
                .filter_map(|line| line.ok())
                .for_each(|line| {
                    let line = format!("[{}] {}", label, line);
                    eprintln!("{}", line);
                });
        })
    };

    let child = Arc::new(Mutex::new(child));

    let join_kill = {
        let child = Arc::clone(&child);
        thread::spawn(move || -> Result<bool, Error> {
            let should_kill = rx_kill.recv().or_else(io_error_from)?;

            if should_kill {
                if let Err(err) = child.lock().or_else(io_error_from)?.kill() {
                    match err.kind() {
                        ErrorKind::InvalidInput | ErrorKind::PermissionDenied => {
                            // child process has exited
                        }
                        _ => {
                            eprintln!("[runp error] failed to kill child process: {}", err);
                            return Err(err);
                        }
                    }
                };

                Ok(true)
            } else {
                Ok(false)
            }
        })
    };

    join_out
        .join()
        .or_else(|_| Err(Error::new(ErrorKind::Other, "stdout pipe failed to join.")))?;
    join_err
        .join()
        .or_else(|_| Err(Error::new(ErrorKind::Other, "stderr pipe failed to join.")))?;

    let status = child.lock().or_else(io_error_from)?.wait()?;

    eprintln!("{}", format!("[{}] {}", label, status));

    Ok((status, join_kill))
}

struct CommandData {
    id: usize,
    command: Command,
    label: String,
    label_display: String,
    kill_receiver: mpsc::Receiver<bool>,
}

type KillSenders = Arc<Mutex<Option<Vec<(usize, mpsc::Sender<bool>)>>>>;

fn kill_all_commands(
    kill_senders: KillSenders,
    exited_command_id: Option<usize>,
) -> Result<(), Error> {
    let mut kill_senders = kill_senders.lock().or_else(io_error_from)?;
    let kill_senders = kill_senders.take();

    if let Some(kill_senders) = kill_senders {
        for (command_id, kill_sender) in kill_senders {
            let should_kill = if let Some(exited_command_id) = exited_command_id {
                command_id != exited_command_id
            } else {
                true
            };

            if let Err(err) = kill_sender.send(should_kill) {
                eprintln!(
                    "[runp error] failed to send kill signal to command: {}",
                    err
                );
            };
        }
    }

    Ok(())
}

pub fn run(config: RunConfig) -> Result<(), Error> {
    let RunConfig {
        commands,
        max_label_length,
        envs,
    } = config;

    let (commands, kill_senders): (Vec<_>, Vec<_>) = commands
        .into_iter()
        .enumerate()
        .map(|(id, cmd)| {
            let (cmd, label) = cmd.into_command_and_label(
                envs.clone()
                    .and_then(|envs| Some(envs.into_iter().collect())),
            );
            let label_display = display_label(&label, max_label_length);

            let (tx, rx) = mpsc::channel();

            (
                CommandData {
                    id,
                    command: cmd,
                    label,
                    label_display,
                    kill_receiver: rx,
                },
                (id, tx),
            )
        })
        .unzip();

    let kill_senders: KillSenders = Arc::new(Mutex::new(Some(kill_senders)));

    let join_labels: Vec<_> = commands
        .into_iter()
        .map(|command| {
            let CommandData {
                command,
                id,
                kill_receiver,
                label,
                label_display,
            } = command;
            let kill_senders = Arc::clone(&kill_senders);
            let join = thread::spawn(move || {
                let ret = command_stdout(command, &label_display, kill_receiver);
                kill_all_commands(kill_senders, Some(id))?;
                ret
            });

            (join, label)
        })
        .collect();

    if let Err(err) = ctrlc::set_handler(move || {
        let kill_senders = Arc::clone(&kill_senders);
        if let Err(err) = kill_all_commands(kill_senders, None) {
            eprintln!(
                "[runp error] failed to kill all commands after Ctrl-C received: {}",
                err
            );
        }
    }) {
        eprintln!(
            "[runp warning] failed to setup Ctrl-C signal handler: {}",
            err
        );
    };

    for (join, label) in join_labels {
        let (_, join_kill) = join.join().or_else(|_| {
            Err(Error::new(
                ErrorKind::Other,
                format!("Failed to join: {}", label),
            ))
        })??;

        let killed = join_kill.join().or_else(|_| {
            Err(Error::new(
                ErrorKind::Other,
                format!("[{}] thread to kill child process failed to join.", label),
            ))
        })??;

        if killed {
            eprintln!("[{}] killed due to other command exited", label);
        }
    }

    Ok(())
}
