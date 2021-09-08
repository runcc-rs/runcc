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

    tokio::select!(
        res = tokio::signal::ctrl_c() => {
            if let Err(err) = res {
                eprintln!(
                    "[runcc][warning] failed to setup Ctrl-C signal handler: {}",
                    err
                );
            } else {
                system.kill_all().await;
                system.wait().await;
            }
        },
        _ = system.wait() => {},
    );

    Ok(())
}
