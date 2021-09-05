use clap::Clap;
use std::io;

use super::options::Opts;

pub fn run() -> io::Result<()> {
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

    crate::run(config)
    // more program logic goes here...
}
