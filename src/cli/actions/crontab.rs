use crate::{crontab, output};
use anyhow::Result;
use tracing::{info, instrument};

/// Execute crontab parsing action
#[instrument(level = "info", fields(verbose = %verbose, color = %color))]
pub fn execute(verbose: bool, color: bool) -> Result<()> {
    info!("Parsing current user's crontab");
    let entries = crontab::parse_current()?;
    output::display_entries(&entries, verbose, color)?;
    Ok(())
}
