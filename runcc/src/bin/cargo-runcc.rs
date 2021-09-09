use runcc::cli::run;

struct ExitMessage(String);

impl std::fmt::Debug for ExitMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[tokio::main]
async fn main() -> Result<(), ExitMessage> {
    if let Err(err) = run().await {
        Err(ExitMessage(format!("{}", err)))
    } else {
        Ok(())
    }
}
