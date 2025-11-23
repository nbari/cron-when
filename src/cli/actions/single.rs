use crate::output;
use anyhow::Result;
use tracing::{info, instrument};

/// Execute a single cron expression
///
/// # Errors
///
/// Returns an error if cron expression parsing or display fails
#[instrument(level = "info", fields(expression = %expression, verbose = %verbose, color = %color))]
pub fn execute(expression: &str, verbose: bool, next: Option<u32>, color: bool) -> Result<()> {
    if let Some(count) = next {
        info!(iterations = count, "Displaying multiple iterations");
        output::display_iterations(expression, count)?;
    } else {
        info!("Displaying single execution time");
        output::display_single(expression, verbose, None, None, color)?;
    }
    Ok(())
}
