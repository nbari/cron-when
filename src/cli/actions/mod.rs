pub mod crontab;
pub mod file;
pub mod single;

use std::path::PathBuf;

/// Represents all possible actions the CLI can perform
#[derive(Debug)]
pub enum Action {
    /// Parse and display a single cron expression
    Single {
        expression: String,
        verbose: bool,
        next: Option<u32>,
        color: bool,
    },
    /// Parse and display crontab from a file
    File {
        path: PathBuf,
        verbose: bool,
        color: bool,
    },
    /// Parse and display current user's crontab
    Crontab { verbose: bool, color: bool },
}
