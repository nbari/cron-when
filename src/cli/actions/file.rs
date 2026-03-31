use crate::{crontab, output};
use anyhow::Result;
use std::path::Path;
use tracing::{info, instrument};

/// Execute file parsing action
///
/// # Errors
///
/// Returns an error if file parsing or display fails
#[instrument(level = "info", fields(path = %path.display(), verbose = %verbose, color = %color))]
pub fn execute(path: &Path, verbose: bool, color: bool) -> Result<()> {
    info!("Parsing crontab file");
    let entries = crontab::parse_file(path)?;
    output::display_entries(&entries, verbose, color)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_file_not_found() {
        let result = execute(Path::new("non_existent_file"), false, false);
        assert!(result.is_err());
    }
}
