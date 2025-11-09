use crate::crontab::CronEntry;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use compound_duration::format_dhms;
use cron_parser::parse;
use tracing::{debug, info, instrument};

/// Display a single cron expression
#[instrument(level = "info", skip(comment), fields(expression = %expression, verbose = %verbose))]
pub fn display_single(expression: &str, verbose: bool, comment: Option<&str>) -> Result<()> {
    let now = Utc::now();
    debug!("Current time: {}", format_datetime(&now));

    // Parse the cron expression and get next execution time
    let next = parse(expression, &now)
        .with_context(|| format!("Failed to parse cron expression: '{}'", expression))?;

    // Calculate duration until next execution
    let duration = next.signed_duration_since(now);
    let seconds = duration.num_seconds().max(0) as u64;

    info!(
        next_execution = %format_datetime(&next),
        seconds_until = %seconds,
        "Calculated next execution time"
    );

    // Format output
    if let Some(comment_text) = comment {
        println!("# {}", comment_text);
    }

    if verbose {
        println!("Cron: {}", expression);
    }

    println!("Next: {}", format_datetime(&next));
    println!("Left: {}", format_dhms(seconds));

    // Add separator for multiple entries
    if comment.is_some() || verbose {
        println!();
    }

    Ok(())
}

/// Display multiple cron entries
#[instrument(level = "info", fields(entry_count = entries.len(), verbose = %verbose))]
pub fn display_entries(entries: &[CronEntry], verbose: bool) -> Result<()> {
    if entries.is_empty() {
        info!("No cron entries to display");
        println!("No valid cron entries found");
        return Ok(());
    }

    debug!("Displaying {} cron entries", entries.len());

    for (i, entry) in entries.iter().enumerate() {
        debug!(index = i, expression = %entry.expression, "Processing entry");
        display_single(&entry.expression, verbose, entry.comment.as_deref())?;
    }

    Ok(())
}

/// Display next N iterations of a cron expression
#[instrument(level = "info", fields(expression = %expression, count = %count))]
pub fn display_iterations(expression: &str, count: u32) -> Result<()> {
    let mut current = Utc::now();

    info!("Calculating {} iterations", count);

    println!("Expression: {}", expression);
    println!();

    for i in 1..=count {
        let next = parse(expression, &current)
            .with_context(|| format!("Failed to parse cron expression: '{}'", expression))?;

        let duration = next.signed_duration_since(Utc::now());
        let seconds = duration.num_seconds().max(0) as u64;

        debug!(iteration = i, next_time = %format_datetime(&next), "Calculated iteration");

        println!(
            "{:3}. {} ({})",
            i,
            format_datetime(&next),
            format_dhms(seconds)
        );

        // Update current time for next iteration
        current = next;
    }

    info!("Completed displaying iterations");

    Ok(())
}

/// Format a DateTime as a human-readable string
fn format_datetime(dt: &DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_datetime() {
        let dt = DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let formatted = format_datetime(&dt);
        assert_eq!(formatted, "2024-01-15 10:30:00 UTC");
    }

    #[test]
    fn test_display_single_valid() {
        let result = display_single("*/5 * * * *", false, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_display_single_invalid() {
        let result = display_single("invalid", false, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_display_entries_empty() {
        let entries = Vec::new();
        let result = display_entries(&entries, false);
        assert!(result.is_ok());
    }
}
