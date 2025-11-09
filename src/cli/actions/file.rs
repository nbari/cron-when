use crate::{crontab, output};
use anyhow::Result;
use std::path::Path;
use tracing::{info, instrument};

/// Execute file parsing action
#[instrument(level = "info", fields(path = %path.display(), verbose = %verbose))]
pub fn execute(path: &Path, verbose: bool) -> Result<()> {
    info!("Parsing crontab file");
    let entries = crontab::parse_file(path)?;
    output::display_entries(&entries, verbose)?;
    Ok(())
}
