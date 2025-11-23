use crate::{crontab, output};
use anyhow::Result;
use tracing::{info, instrument};

/// Execute crontab parsing action
///
/// # Errors
///
/// Returns an error if crontab parsing or display fails
#[instrument(level = "info", fields(verbose = %verbose, color = %color))]
pub fn execute(verbose: bool, color: bool) -> Result<()> {
    info!("Parsing current user's crontab");
    let entries = crontab::parse_current()?;
    output::display_entries(&entries, verbose, color)?;
    Ok(())
}
