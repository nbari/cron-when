use crate::{crontab, output};
use anyhow::Result;
use tracing::{info, instrument};

/// Execute crontab parsing action
#[instrument(level = "info", fields(verbose = %verbose))]
pub fn execute(verbose: bool) -> Result<()> {
    info!("Parsing current user's crontab");
    let entries = crontab::parse_current()?;
    output::display_entries(&entries, verbose)?;
    Ok(())
}
