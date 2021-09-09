use runcc::cli::run;

struct ExitMessage(String);

impl std::fmt::Debug for ExitMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[tokio::main]
async fn main() -> Result<(), ExitMessage> {
    let exit_code: i32 = match run().await {
        Err(err) => return Err(ExitMessage(format!("{}", err))),
        Ok(report) => {
            let failed = report.command_count_failed();
            if failed == 0 {
                return Ok(());
            } else {
                2
            }
        }
    };

    std::process::exit(exit_code);
}
