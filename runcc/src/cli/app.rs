use clap::Clap;
use std::io;

use super::{options::Opts, CommandSystemLogPlugin};
use crate::run::CommandSystemSimpleReport;

pub async fn run() -> io::Result<CommandSystemSimpleReport> {
    let args = std::env::args_os();
    let mut args: Vec<_> = args.collect();

    if let Some(arg) = args.get(1) {
        if arg == "runcc" {
            args.remove(1);
        }
    }

    let opts: Opts = Opts::parse_from(args);

    let config = opts.try_into_config().or_else(|err| {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{}", err),
        ))
    })?;

    let mut system =
        crate::run::spawn_from_run_config_with_plugin(config, CommandSystemLogPlugin::new());

    let killer = system.share_killer();

    tokio::spawn(async move {
        if let Err(err) = tokio::signal::ctrl_c().await {
            eprintln!(
                "[runcc][warning] failed to setup Ctrl-C signal handler: {}",
                err
            );
        } else {
            killer.kill_all().await;
        }
    });

    Ok(system.wait().await)
}
