use crate::crontab::{CronEntry, ScheduleExpression, normalized_schedule_expression};
use anyhow::{Context, Result};
use chrono::{DateTime, Local, Utc};
use colored::Colorize;
use compound_duration::format_dhms;
use cron_parser::parse;
use std::io::Write;
use tracing::{debug, info, instrument};

/// Display a single cron expression
///
/// # Errors
///
/// Returns an error if the cron expression cannot be parsed
#[instrument(level = "info", skip(comment, command), fields(expression = %expression, verbose = %verbose, color = %color))]
pub fn display_single(
    expression: &str,
    verbose: bool,
    comment: Option<&str>,
    command: Option<&str>,
    color: bool,
) -> Result<()> {
    display_single_with_writer(
        &mut std::io::stdout(),
        expression,
        verbose,
        comment,
        command,
        color,
    )
}

/// Display a single cron expression to a specific writer
///
/// # Errors
///
/// Returns an error if the cron expression cannot be parsed or writing fails
pub fn display_single_with_writer(
    writer: &mut impl Write,
    expression: &str,
    _verbose: bool,
    comment: Option<&str>,
    command: Option<&str>,
    color: bool,
) -> Result<()> {
    // Force colors on if explicitly requested, regardless of TTY detection
    if color {
        colored::control::set_override(true);
    }

    let schedule = normalized_schedule_expression(expression)
        .with_context(|| format!("Failed to parse cron expression: '{expression}'"))?;

    if schedule == ScheduleExpression::Reboot {
        return display_reboot(writer, expression, comment, command, color);
    }

    let now = Utc::now();
    let formatted_now = format_datetime(&now);
    debug!("Current time: {formatted_now}");

    let ScheduleExpression::Standard(schedule_expression) = schedule else {
        unreachable!("reboot expressions return early");
    };

    // Parse the cron expression and get next execution time
    let next = parse(schedule_expression, &now)
        .with_context(|| format!("Failed to parse cron expression: '{expression}'"))?;

    // Calculate duration until next execution
    let duration = next.signed_duration_since(now);
    let seconds = u64::try_from(duration.num_seconds().max(0)).unwrap_or(0);

    info!(
        next_execution = %format_datetime(&next),
        seconds_until = %seconds,
        "Calculated next execution time"
    );

    // Format output
    write_comment(writer, comment, color)?;

    // Always show cron expression with quotes for clarity
    if color {
        writeln!(
            writer,
            "{} \"{}\"",
            "Cron:".green().bold(),
            expression.yellow()
        )?;
    } else {
        writeln!(writer, "Cron: \"{expression}\"")?;
    }

    // Show command if available
    if let Some(cmd) = command {
        if color {
            writeln!(writer, "{} {}", "Command:".red().bold(), cmd.white())?;
        } else {
            writeln!(writer, "Command: {cmd}")?;
        }
    }

    if color {
        writeln!(
            writer,
            "{} {}",
            "Next:".blue().bold(),
            format_datetime(&next)
        )?;
        writeln!(
            writer,
            "{} {}",
            "Left:".yellow().bold(),
            format_dhms(seconds)
        )?;
    } else {
        writeln!(writer, "Next: {}", format_datetime(&next))?;
        writeln!(writer, "Left: {}", format_dhms(seconds))?;
    }

    // Add separator for multiple entries
    writeln!(writer)?;

    Ok(())
}

/// Helper to display @reboot entries
fn display_reboot(
    writer: &mut impl Write,
    expression: &str,
    comment: Option<&str>,
    command: Option<&str>,
    color: bool,
) -> Result<()> {
    write_comment(writer, comment, color)?;
    if color {
        writeln!(
            writer,
            "{} \"{}\"",
            "Cron:".green().bold(),
            expression.yellow()
        )?;
    } else {
        writeln!(writer, "Cron: \"{expression}\"")?;
    }
    if let Some(cmd) = command {
        if color {
            writeln!(writer, "{} {}", "Command:".red().bold(), cmd.white())?;
        } else {
            writeln!(writer, "Command: {cmd}")?;
        }
    }
    if color {
        writeln!(writer, "{} System Startup", "Next:".blue().bold())?;
        writeln!(writer, "{} N/A", "Left:".yellow().bold())?;
    } else {
        writeln!(writer, "Next: System Startup")?;
        writeln!(writer, "Left: N/A")?;
    }
    writeln!(writer)?;
    Ok(())
}

fn write_comment(writer: &mut impl Write, comment: Option<&str>, color: bool) -> Result<()> {
    let Some(comment) = comment else {
        return Ok(());
    };

    for line in comment.lines() {
        if color {
            writeln!(writer, "{}", format!("# {line}").bright_black())?;
        } else {
            writeln!(writer, "# {line}")?;
        }
    }

    Ok(())
}

/// Display multiple cron entries
///
/// # Errors
///
/// Returns an error if any cron expression cannot be parsed
#[instrument(level = "info", fields(entry_count = entries.len(), verbose = %verbose, color = %color))]
pub fn display_entries(entries: &[CronEntry], verbose: bool, color: bool) -> Result<()> {
    display_entries_with_writer(&mut std::io::stdout(), entries, verbose, color)
}

/// Display multiple cron entries to a specific writer
///
/// # Errors
///
/// Returns an error if any cron expression cannot be parsed or writing fails
pub fn display_entries_with_writer(
    writer: &mut impl Write,
    entries: &[CronEntry],
    verbose: bool,
    color: bool,
) -> Result<()> {
    // Force colors on if explicitly requested, regardless of TTY detection
    if color {
        colored::control::set_override(true);
    }

    if entries.is_empty() {
        info!("No cron entries to display");
        writeln!(writer, "No valid cron entries found")?;
        return Ok(());
    }

    debug!("Displaying {} cron entries", entries.len());

    for (i, entry) in entries.iter().enumerate() {
        debug!(index = i, expression = %entry.expression, "Processing entry");
        display_single_with_writer(
            writer,
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
///
/// # Errors
///
/// Returns an error if the cron expression cannot be parsed
#[instrument(level = "info", fields(expression = %expression, count = %count))]
pub fn display_iterations(expression: &str, count: u32) -> Result<()> {
    display_iterations_with_writer(&mut std::io::stdout(), expression, count)
}

/// Display next N iterations of a cron expression to a specific writer
///
/// # Errors
///
/// Returns an error if the cron expression cannot be parsed or writing fails
pub fn display_iterations_with_writer(
    writer: &mut impl Write,
    expression: &str,
    count: u32,
) -> Result<()> {
    let schedule = normalized_schedule_expression(expression)
        .with_context(|| format!("Failed to parse cron expression: '{expression}'"))?;

    if schedule == ScheduleExpression::Reboot {
        writeln!(writer, "Expression: {expression}")?;
        writeln!(writer)?;
        writeln!(writer, "  1. System Startup (Runs once at boot)")?;
        return Ok(());
    }

    let ScheduleExpression::Standard(schedule_expression) = schedule else {
        unreachable!("reboot expressions return early");
    };

    let mut current = Utc::now();

    info!("Calculating {count} iterations");

    writeln!(writer, "Expression: {expression}")?;
    writeln!(writer)?;

    for i in 1..=count {
        let next = parse(schedule_expression, &current)
            .with_context(|| format!("Failed to parse cron expression: '{expression}'"))?;

        let duration = next.signed_duration_since(Utc::now());
        let seconds = u64::try_from(duration.num_seconds().max(0)).unwrap_or(0);

        debug!(iteration = i, next_time = %format_datetime(&next), "Calculated iteration");

        writeln!(
            writer,
            "{:3}. {} ({})",
            i,
            format_datetime(&next),
            format_dhms(seconds)
        )?;

        // Update current time for next iteration
        current = next;
    }

    info!("Completed displaying iterations");

    Ok(())
}

/// Format a `DateTime` as a human-readable string with both UTC and local timezone
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
        let Ok(dt) = DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z") else {
            return;
        };
        let dt = dt.with_timezone(&Utc);
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
            parts.first().is_some_and(|p| p.ends_with("UTC ")),
            "First part should end with 'UTC '"
        );
        assert!(
            parts.get(1).is_some_and(|p| p.ends_with(')')),
            "Second part should end with ')'"
        );
    }

    #[test]
    fn test_format_datetime_includes_timezone() {
        let Ok(dt) = DateTime::parse_from_rfc3339("2024-06-15T14:30:00Z") else {
            return;
        };
        let dt = dt.with_timezone(&Utc);
        let formatted = format_datetime(&dt);

        // Should have the pattern: YYYY-MM-DD HH:MM:SS UTC (HH:MM:SS TZ)
        assert!(formatted.starts_with("2024-06-15 14:30:00 UTC"));

        // The local time part should be in parentheses
        let local_part_start = formatted.find('(');
        assert!(
            local_part_start.is_some(),
            "Should have opening parenthesis"
        );
        let local_part_start = local_part_start.unwrap_or_default();

        let local_part_end = formatted.find(')');
        assert!(local_part_end.is_some(), "Should have closing parenthesis");
        let local_part_end = local_part_end.unwrap_or_default();
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
            local_parts.first().map_or(0, |p| p.split(':').count()),
            3,
            "Time should have hours, minutes, seconds"
        );
    }

    #[test]
    fn test_display_single_with_writer() -> Result<()> {
        let mut buf = Vec::new();
        display_single_with_writer(
            &mut buf,
            "*/5 * * * *",
            false,
            Some("test comment"),
            Some("test command"),
            false,
        )?;

        let output = String::from_utf8(buf).context("Failed to parse output as UTF-8")?;
        assert!(output.contains("# test comment"));
        assert!(output.contains("Cron: \"*/5 * * * *\""));
        assert!(output.contains("Command: test command"));
        assert!(output.contains("Next:"));
        assert!(output.contains("Left:"));
        Ok(())
    }

    #[test]
    fn test_display_single_preserves_alias_in_output() -> Result<()> {
        let mut buf = Vec::new();
        display_single_with_writer(&mut buf, "@daily", false, None, Some("/bin/true"), false)?;

        let output = String::from_utf8(buf).context("Failed to parse output as UTF-8")?;
        assert!(output.contains("Cron: \"@daily\""));
        assert!(output.contains("Command: /bin/true"));
        Ok(())
    }

    #[test]
    fn test_display_single_renders_multiline_comments_line_by_line() -> Result<()> {
        let mut buf = Vec::new();
        display_single_with_writer(
            &mut buf,
            "0 * * * *",
            false,
            Some("first\nsecond"),
            None,
            false,
        )?;

        let output = String::from_utf8(buf).context("Failed to parse output as UTF-8")?;
        assert!(output.contains("# first\n# second\n"));
        Ok(())
    }

    #[test]
    fn test_display_entries_with_writer_preserves_alias_and_comment_formatting() -> Result<()> {
        let entries = vec![CronEntry {
            expression: "@hourly".to_string(),
            command: Some("/usr/bin/backup.sh".to_string()),
            comment: Some("first\nsecond".to_string()),
        }];

        let mut buf = Vec::new();
        display_entries_with_writer(&mut buf, &entries, false, false)?;

        let output = String::from_utf8(buf).context("Failed to parse output as UTF-8")?;
        assert!(output.contains("# first\n# second\n"));
        assert!(output.contains("Cron: \"@hourly\""));
        assert!(output.contains("Command: /usr/bin/backup.sh"));
        Ok(())
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
    fn test_display_iterations_reboot() -> Result<()> {
        let mut buf = Vec::new();
        display_iterations_with_writer(&mut buf, "@reboot", 5)?;

        let output = String::from_utf8(buf).context("Failed to parse output as UTF-8")?;
        assert!(output.contains("Expression: @reboot"));
        assert!(output.contains("System Startup"));
        Ok(())
    }

    #[test]
    fn test_display_iterations_preserve_alias_in_header() -> Result<()> {
        let mut buf = Vec::new();
        display_iterations_with_writer(&mut buf, "@daily", 1)?;

        let output = String::from_utf8(buf).context("Failed to parse output as UTF-8")?;
        assert!(output.contains("Expression: @daily"));
        assert!(output.contains("  1. "));
        Ok(())
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
            "@daily",         // Daily alias
            "@hourly",        // Hourly alias
            "@reboot",        // Reboot alias
        ];

        for expr in expressions {
            let result = display_single(expr, false, None, None, false);
            assert!(result.is_ok(), "Failed to parse cron expression: {expr}");
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
            assert!(result.is_err(), "Should have failed for: {expr}");
        }
    }
}
