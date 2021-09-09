#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct CommandSystemSimpleReport {
    pub command_count_total: usize,
    pub command_count_success: usize,
}

impl CommandSystemSimpleReport {
    pub fn command_count_failed(&self) -> usize {
        self.command_count_total - self.command_count_success
    }
}
