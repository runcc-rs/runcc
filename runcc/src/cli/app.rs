use clap::Clap;
use std::io;

use super::{options::Opts, CommandSystemLogPlugin};

pub async fn run() -> io::Result<()> {
    let opts: Opts = Opts::parse();

    let config = opts
        .try_into_config()
        .or_else(|err| {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{}", err),
            ))
        })?
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "Please specify commands from cli args or use a config file",
            )
        })?;

    let mut system =
        crate::run::spawn_from_run_config_with_plugin(config, CommandSystemLogPlugin::new())?;

    let killer = system.share_killer();

    let _ = tokio::spawn(async move {
        if let Err(err) = tokio::signal::ctrl_c().await {
            eprintln!(
                "[runcc][warning] failed to setup Ctrl-C signal handler{}",
                err
            )
        };

        killer.kill_all().await;
    });

    system.wait_into_stopped_commands().await;

    Ok(())
}
