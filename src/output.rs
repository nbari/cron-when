use crate::crontab::CronEntry;
use anyhow::{Context, Result};
use chrono::{DateTime, Local, Utc};
use colored::*;
use compound_duration::format_dhms;
use cron_parser::parse;
use tracing::{debug, info, instrument};

/// Display a single cron expression
#[instrument(level = "info", skip(comment, command), fields(expression = %expression, verbose = %verbose, color = %color))]
pub fn display_single(
    expression: &str,
    verbose: bool,
    comment: Option<&str>,
    command: Option<&str>,
    color: bool,
) -> Result<()> {
    // Force colors on if explicitly requested, regardless of TTY detection
    if color {
        colored::control::set_override(true);
    }

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
        if color {
            println!("{}", format!("# {}", comment_text).bright_black());
        } else {
            println!("# {}", comment_text);
        }
    }

    // Always show cron expression with quotes for clarity
    if color {
        println!("{} \"{}\"", "Cron:".green().bold(), expression.yellow());
    } else {
        println!("Cron: \"{}\"", expression);
    }

    // Show command if available
    if let Some(cmd) = command {
        if color {
            println!("{} {}", "Command:".red().bold(), cmd.white());
        } else {
            println!("Command: {}", cmd);
        }
    }

    if color {
        println!("{} {}", "Next:".blue().bold(), format_datetime(&next));
        println!("{} {}", "Left:".yellow().bold(), format_dhms(seconds));
    } else {
        println!("Next: {}", format_datetime(&next));
        println!("Left: {}", format_dhms(seconds));
    }

    // Add separator for multiple entries
    println!();

    Ok(())
}

/// Display multiple cron entries
#[instrument(level = "info", fields(entry_count = entries.len(), verbose = %verbose, color = %color))]
pub fn display_entries(entries: &[CronEntry], verbose: bool, color: bool) -> Result<()> {
    // Force colors on if explicitly requested, regardless of TTY detection
    if color {
        colored::control::set_override(true);
    }

    if entries.is_empty() {
        info!("No cron entries to display");
        println!("No valid cron entries found");
        return Ok(());
    }

    debug!("Displaying {} cron entries", entries.len());

    for (i, entry) in entries.iter().enumerate() {
        debug!(index = i, expression = %entry.expression, "Processing entry");
        display_single(
            &entry.expression,
            verbose,
            entry.comment.as_deref(),
            entry.command.as_deref(),
            color,
        )?;
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

/// Format a DateTime as a human-readable string with both UTC and local timezone
fn format_datetime(dt: &DateTime<Utc>) -> String {
    let local = dt.with_timezone(&Local);
    format!(
        "{} ({} {})",
        dt.format("%Y-%m-%d %H:%M:%S UTC"),
        local.format("%H:%M:%S"),
        local.format("%Z")
    )
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

        // Check that it contains UTC time in the correct format
        assert!(formatted.contains("2024-01-15 10:30:00 UTC"));

        // Check that it contains local time in parentheses
        assert!(formatted.contains('('));
        assert!(formatted.contains(')'));

        // Verify the format has three parts: UTC datetime, local time, and timezone
        let parts: Vec<&str> = formatted.split('(').collect();
        assert_eq!(
            parts.len(),
            2,
            "Format should have UTC and local time parts"
        );
        assert!(
            parts[0].ends_with("UTC "),
            "First part should end with 'UTC '"
        );
        assert!(parts[1].ends_with(')'), "Second part should end with ')'");
    }

    #[test]
    fn test_format_datetime_includes_timezone() {
        let dt = DateTime::parse_from_rfc3339("2024-06-15T14:30:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let formatted = format_datetime(&dt);

        // Should have the pattern: YYYY-MM-DD HH:MM:SS UTC (HH:MM:SS TZ)
        assert!(formatted.starts_with("2024-06-15 14:30:00 UTC"));

        // The local time part should be in parentheses
        let local_part_start = formatted
            .find('(')
            .expect("Should have opening parenthesis");
        let local_part_end = formatted
            .find(')')
            .expect("Should have closing parenthesis");
        assert!(local_part_end > local_part_start);

        // Extract local time part and verify it has time and timezone
        let local_part = &formatted[local_part_start + 1..local_part_end];
        let local_parts: Vec<&str> = local_part.split_whitespace().collect();
        assert_eq!(
            local_parts.len(),
            2,
            "Local part should have time and timezone"
        );

        // Verify time format HH:MM:SS
        assert_eq!(
            local_parts[0].split(':').count(),
            3,
            "Time should have hours, minutes, seconds"
        );
    }

    #[test]
    fn test_display_single_valid() {
        let result = display_single("*/5 * * * *", false, None, None, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_display_single_valid_verbose() {
        // Verbose parameter should not affect output anymore
        let result = display_single("*/5 * * * *", true, None, None, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_display_single_with_comment() {
        let result = display_single("0 * * * *", false, Some("Run every hour"), None, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_display_single_invalid() {
        let result = display_single("invalid", false, None, None, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_display_single_invalid_expression_with_comment() {
        let result = display_single("not a cron", false, Some("This will fail"), None, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_display_entries_empty() {
        let entries = Vec::new();
        let result = display_entries(&entries, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_display_entries_single() {
        let entries = vec![CronEntry {
            expression: "0 * * * *".to_string(),
            command: None,
            comment: None,
        }];
        let result = display_entries(&entries, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_display_entries_multiple() {
        let entries = vec![
            CronEntry {
                expression: "0 * * * *".to_string(),
                command: Some("/usr/bin/backup.sh".to_string()),
                comment: Some("Hourly backup".to_string()),
            },
            CronEntry {
                expression: "0 0 * * *".to_string(),
                command: Some("/usr/bin/cleanup.sh".to_string()),
                comment: Some("Daily cleanup".to_string()),
            },
            CronEntry {
                expression: "*/15 * * * *".to_string(),
                command: None,
                comment: None,
            },
        ];
        let result = display_entries(&entries, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_display_entries_with_invalid() {
        let entries = vec![
            CronEntry {
                expression: "0 * * * *".to_string(),
                command: None,
                comment: None,
            },
            CronEntry {
                expression: "invalid cron".to_string(),
                command: None,
                comment: None,
            },
        ];
        // Should fail on the invalid entry
        let result = display_entries(&entries, false, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_display_iterations_valid() {
        let result = display_iterations("0 12 * * *", 3);
        assert!(result.is_ok());
    }

    #[test]
    fn test_display_iterations_single() {
        let result = display_iterations("*/5 * * * *", 1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_display_iterations_invalid() {
        let result = display_iterations("not valid", 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_display_iterations_many() {
        let result = display_iterations("0 0 * * 0", 10);
        assert!(result.is_ok());
    }

    #[test]
    fn test_various_cron_expressions() {
        // Test various valid cron expressions
        let expressions = vec![
            "* * * * *",      // Every minute
            "0 * * * *",      // Every hour
            "0 0 * * *",      // Daily at midnight
            "0 0 * * 0",      // Weekly on Sunday
            "0 0 1 * *",      // Monthly on 1st
            "*/15 * * * *",   // Every 15 minutes
            "0 9-17 * * 1-5", // Weekdays 9am-5pm
            "30 2 * * *",     // Daily at 2:30am
            "0 */2 * * *",    // Every 2 hours
            "0 0 1 1 *",      // Yearly on Jan 1st
        ];

        for expr in expressions {
            let result = display_single(expr, false, None, None, false);
            assert!(result.is_ok(), "Failed to parse cron expression: {}", expr);
        }
    }

    #[test]
    fn test_various_invalid_expressions() {
        // Test various invalid cron expressions
        let invalid_expressions = vec![
            "",
            "invalid",
            "* * * *",     // Too few fields
            "* * * * * *", // Too many fields
            "60 * * * *",  // Invalid minute
            "* 24 * * *",  // Invalid hour
            "* * 32 * *",  // Invalid day
            "* * * 13 *",  // Invalid month
            "* * * * 7",   // Invalid day of week
            "xyz * * * *", // Non-numeric
        ];

        for expr in invalid_expressions {
            let result = display_single(expr, false, None, None, false);
            assert!(result.is_err(), "Should have failed for: {}", expr);
        }
    }
}
