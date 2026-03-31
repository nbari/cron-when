use anyhow::{Context, Result};
use chrono::Utc;
use cron_parser::parse;
use std::fs;
use std::path::Path;
use std::process::Command;
use tracing::{debug, info, instrument, warn};

#[derive(Debug, Clone)]
pub struct CronEntry {
    pub expression: String,
    pub command: Option<String>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScheduleExpression<'a> {
    Reboot,
    Standard(&'a str),
}

/// Parse crontab from current user
///
/// # Errors
///
/// Returns an error if crontab command execution or parsing fails
#[instrument(level = "info")]
pub fn parse_current() -> Result<Vec<CronEntry>> {
    debug!("Executing 'crontab -l' command");

    let output = Command::new("crontab")
        .arg("-l")
        .output()
        .context("Failed to execute 'crontab -l'. Make sure crontab is installed.")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("no crontab") {
            info!("No crontab found for current user");
            return Ok(Vec::new());
        }
        anyhow::bail!("crontab -l failed: {stderr}");
    }

    let content =
        String::from_utf8(output.stdout).context("Failed to parse crontab output as UTF-8")?;

    let entries = parse_content(&content);
    info!(entry_count = entries.len(), "Parsed crontab entries");

    Ok(entries)
}

/// Parse crontab from file
///
/// # Errors
///
/// Returns an error if file reading or parsing fails
#[instrument(level = "info", fields(path = %path.display()))]
pub fn parse_file(path: &Path) -> Result<Vec<CronEntry>> {
    debug!("Reading crontab file");

    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {path}", path = path.display()))?;

    let entries = parse_content(&content);
    info!(entry_count = entries.len(), "Parsed crontab file entries");

    Ok(entries)
}

/// Parse crontab content and extract entries
#[instrument(level = "debug", skip(content))]
fn parse_content(content: &str) -> Vec<CronEntry> {
    let mut entries = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut current_comments = Vec::new();
    let now = Utc::now();

    debug!(line_count = lines.len(), "Parsing crontab content");

    for line in lines {
        let trimmed = line.trim();

        // Skip empty lines and collect comments
        if trimmed.is_empty() {
            current_comments.clear();
            continue;
        }

        if trimmed.starts_with('#') {
            current_comments.push(trimmed.trim_start_matches('#').trim().to_string());
            continue;
        }

        // Skip environment variable assignments
        if is_env_var(trimmed) {
            debug!(line = trimmed, "Skipping environment variable");
            current_comments.clear();
            continue;
        }

        // Try to parse as cron entry
        if let Some((expression, command, inline_comment)) = extract_cron_entry(trimmed) {
            let Some(schedule) = normalized_schedule_expression(&expression) else {
                warn!(line = trimmed, "Unsupported cron alias");
                current_comments.clear();
                continue;
            };

            if let ScheduleExpression::Standard(schedule_expression) = schedule
                && parse(schedule_expression, &now).is_err()
            {
                warn!(line = trimmed, expression = %expression, "Invalid cron expression");
                current_comments.clear();
                continue;
            }

            let comment = merge_comments(&current_comments, inline_comment);
            debug!(expression = %expression, has_comment = comment.is_some(), "Found cron entry");
            entries.push(CronEntry {
                expression,
                command: Some(command),
                comment,
            });
            current_comments.clear();
        } else {
            warn!(line = trimmed, "Could not parse as cron expression");
            current_comments.clear();
        }
    }

    entries
}

/// Check if line is an environment variable assignment
fn is_env_var(line: &str) -> bool {
    if let Some((key, _)) = line.split_once('=') {
        return !key.contains(char::is_whitespace);
    }
    false
}

/// Extract cron expression and command from a line
fn extract_cron_entry(line: &str) -> Option<(String, String, Option<String>)> {
    let (expression, command_tail) = split_expression_and_command(line)?;
    let (command, inline_comment) = split_command_and_inline_comment(command_tail);
    let command = command.trim();

    if command.is_empty() {
        return None;
    }

    Some((expression, command.to_string(), inline_comment))
}

fn split_expression_and_command(line: &str) -> Option<(String, &str)> {
    let trimmed = line.trim();
    let mut fields = trimmed.split_whitespace();
    let first = fields.next()?;

    if first.starts_with('@') {
        let alias_end = first.len();
        let command_tail = trimmed.get(alias_end..)?.trim_start();
        return Some((first.to_string(), command_tail));
    }

    let mut field_count = 1usize;
    let mut end_index = first.len();

    while field_count < 5 {
        let remainder = trimmed.get(end_index..)?;
        let whitespace_len = remainder
            .chars()
            .take_while(|ch| ch.is_whitespace())
            .map(char::len_utf8)
            .sum::<usize>();

        if whitespace_len == 0 {
            return None;
        }

        end_index += whitespace_len;

        let remainder = trimmed.get(end_index..)?;
        let field_len = remainder
            .chars()
            .take_while(|ch| !ch.is_whitespace())
            .map(char::len_utf8)
            .sum::<usize>();

        if field_len == 0 {
            return None;
        }

        end_index += field_len;
        field_count += 1;
    }

    let expression = trimmed.get(..end_index)?.to_string();
    let command_tail = trimmed.get(end_index..)?.trim_start();
    Some((expression, command_tail))
}

fn split_command_and_inline_comment(command: &str) -> (&str, Option<String>) {
    let mut in_single_quotes = false;
    let mut in_double_quotes = false;
    let mut escaped = false;
    let mut previous = None;

    for (index, ch) in command.char_indices() {
        if escaped {
            escaped = false;
            previous = Some(ch);
            continue;
        }

        match ch {
            '\\' => {
                escaped = true;
            }
            '\'' if !in_double_quotes => {
                in_single_quotes = !in_single_quotes;
            }
            '"' if !in_single_quotes => {
                in_double_quotes = !in_double_quotes;
            }
            '#' if !in_single_quotes
                && !in_double_quotes
                && previous.is_some_and(char::is_whitespace) =>
            {
                let command_part = command[..index].trim_end();
                let inline_comment = command[index + ch.len_utf8()..].trim();
                let inline_comment = if inline_comment.is_empty() {
                    None
                } else {
                    Some(inline_comment.to_string())
                };

                return (command_part, inline_comment);
            }
            _ => {}
        }

        previous = Some(ch);
    }

    (command.trim_end(), None)
}

pub(crate) fn normalized_schedule_expression(expression: &str) -> Option<ScheduleExpression<'_>> {
    match expression {
        "@reboot" => Some(ScheduleExpression::Reboot),
        "@yearly" | "@annually" => Some(ScheduleExpression::Standard("0 0 1 1 *")),
        "@monthly" => Some(ScheduleExpression::Standard("0 0 1 * *")),
        "@weekly" => Some(ScheduleExpression::Standard("0 0 * * 0")),
        "@daily" | "@midnight" => Some(ScheduleExpression::Standard("0 0 * * *")),
        "@hourly" => Some(ScheduleExpression::Standard("0 * * * *")),
        _ if expression.starts_with('@') => None,
        _ => Some(ScheduleExpression::Standard(expression)),
    }
}

fn merge_comments(block_comments: &[String], inline_comment: Option<String>) -> Option<String> {
    let mut comments = block_comments.to_vec();
    if let Some(inline_comment) = inline_comment {
        comments.push(inline_comment);
    }

    if comments.is_empty() {
        None
    } else {
        Some(comments.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_content() {
        let content = r"
# Run every 5 minutes
*/5 * * * * /usr/bin/script1.sh

# Daily backup at midnight
0 0 * * * /usr/bin/backup.sh

# Environment variable
SHELL=/bin/bash

# Another comment without entry

30 2 * * 1 /usr/bin/weekly.sh
";

        let entries = parse_content(content);
        assert_eq!(entries.len(), 3);

        assert_eq!(
            entries.first().map(|e| e.expression.as_str()),
            Some("*/5 * * * *")
        );
        assert_eq!(
            entries.first().and_then(|e| e.command.as_deref()),
            Some("/usr/bin/script1.sh")
        );
        assert_eq!(
            entries.first().and_then(|e| e.comment.as_deref()),
            Some("Run every 5 minutes")
        );

        assert_eq!(
            entries.get(1).map(|e| e.expression.as_str()),
            Some("0 0 * * *")
        );
        assert_eq!(
            entries.get(1).and_then(|e| e.command.as_deref()),
            Some("/usr/bin/backup.sh")
        );
        assert_eq!(
            entries.get(1).and_then(|e| e.comment.as_deref()),
            Some("Daily backup at midnight")
        );

        assert_eq!(
            entries.get(2).map(|e| e.expression.as_str()),
            Some("30 2 * * 1")
        );
        assert_eq!(
            entries.get(2).and_then(|e| e.command.as_deref()),
            Some("/usr/bin/weekly.sh")
        );
        assert_eq!(entries.get(2).and_then(|e| e.comment.as_deref()), None);
    }

    #[test]
    fn test_parse_content_complex() {
        let content = r"
# First comment line
# Second comment line
*/5 * * * * /usr/bin/script1.sh

# This is a daily job
@daily /usr/bin/daily.sh

# Job with inline comment
0 0 * * * /usr/bin/backup.sh # backup now
";

        let entries = parse_content(content);

        assert_eq!(entries.len(), 3);

        // Check first entry (multi-line comment)
        assert_eq!(
            entries.first().map(|e| e.expression.as_str()),
            Some("*/5 * * * *")
        );
        assert_eq!(
            entries.first().and_then(|e| e.comment.as_deref()),
            Some("First comment line\nSecond comment line")
        );

        // Check second entry (alias)
        assert_eq!(
            entries.get(1).map(|e| e.expression.as_str()),
            Some("@daily")
        );
        assert_eq!(
            entries.get(1).and_then(|e| e.command.as_deref()),
            Some("/usr/bin/daily.sh")
        );
        assert_eq!(
            entries.get(1).and_then(|e| e.comment.as_deref()),
            Some("This is a daily job")
        );

        // Check third entry (inline comment merged into comment block)
        assert_eq!(
            entries.get(2).map(|e| e.expression.as_str()),
            Some("0 0 * * *")
        );
        assert_eq!(
            entries.get(2).and_then(|e| e.command.as_deref()),
            Some("/usr/bin/backup.sh")
        );
        assert_eq!(
            entries.get(2).and_then(|e| e.comment.as_deref()),
            Some("Job with inline comment\nbackup now")
        );
    }

    #[test]
    fn test_parse_content_alias_with_inline_comment() {
        let content = "@hourly /usr/bin/backup.sh # rotate logs\n";

        let entries = parse_content(content);
        assert_eq!(entries.len(), 1);
        assert_eq!(
            entries.first().map(|entry| entry.expression.as_str()),
            Some("@hourly")
        );
        assert_eq!(
            entries.first().and_then(|entry| entry.command.as_deref()),
            Some("/usr/bin/backup.sh")
        );
        assert_eq!(
            entries.first().and_then(|entry| entry.comment.as_deref()),
            Some("rotate logs")
        );
    }

    #[test]
    fn test_parse_content_preserves_literal_hash_in_command() {
        let content = "0 0 * * * echo foo#bar\n";

        let entries = parse_content(content);
        assert_eq!(entries.len(), 1);
        assert_eq!(
            entries.first().and_then(|entry| entry.command.as_deref()),
            Some("echo foo#bar")
        );
        assert_eq!(
            entries.first().and_then(|entry| entry.comment.as_deref()),
            None
        );
    }

    #[test]
    fn test_parse_content_splits_inline_comment_when_preceded_by_whitespace() {
        let content = "0 0 * * * echo foo # backup\n";

        let entries = parse_content(content);
        assert_eq!(entries.len(), 1);
        assert_eq!(
            entries.first().and_then(|entry| entry.command.as_deref()),
            Some("echo foo")
        );
        assert_eq!(
            entries.first().and_then(|entry| entry.comment.as_deref()),
            Some("backup")
        );
    }

    #[test]
    fn test_parse_content_preserves_escaped_and_quoted_hashes() {
        let content = r#"
0 0 * * * echo foo\#bar
0 0 * * * echo "foo # not-comment"
0 0 * * * echo 'foo # still-not-comment'
"#;

        let entries = parse_content(content);
        assert_eq!(entries.len(), 3);
        assert_eq!(
            entries.first().and_then(|entry| entry.command.as_deref()),
            Some(r"echo foo\#bar")
        );
        assert_eq!(
            entries.get(1).and_then(|entry| entry.command.as_deref()),
            Some(r#"echo "foo # not-comment""#)
        );
        assert_eq!(
            entries.get(2).and_then(|entry| entry.command.as_deref()),
            Some("echo 'foo # still-not-comment'")
        );
    }

    #[test]
    fn test_parse_content_merges_block_and_inline_comments() {
        let content = r"
# first
# second
0 0 * * * /bin/true # third
";

        let entries = parse_content(content);
        assert_eq!(entries.len(), 1);
        assert_eq!(
            entries.first().and_then(|entry| entry.comment.as_deref()),
            Some("first\nsecond\nthird")
        );
    }

    #[test]
    fn test_parse_content_skips_alias_without_command() {
        let content = "@daily\n";

        let entries = parse_content(content);
        assert!(entries.is_empty());
    }

    #[test]
    fn test_parse_content_skips_unknown_alias_without_aborting() {
        let content = r"
@unknown /bin/true
0 0 * * * /bin/echo ok
";

        let entries = parse_content(content);
        assert_eq!(entries.len(), 1);
        assert_eq!(
            entries.first().map(|entry| entry.expression.as_str()),
            Some("0 0 * * *")
        );
    }

    #[test]
    fn test_parse_content_skips_invalid_standard_expression_without_aborting() {
        let content = r"
61 * * * * /bin/false
0 0 * * * /bin/echo ok
";

        let entries = parse_content(content);
        assert_eq!(entries.len(), 1);
        assert_eq!(
            entries.first().and_then(|entry| entry.command.as_deref()),
            Some("/bin/echo ok")
        );
    }

    #[test]
    fn test_normalized_schedule_expression() {
        assert_eq!(
            normalized_schedule_expression("@daily"),
            Some(ScheduleExpression::Standard("0 0 * * *"))
        );
        assert_eq!(
            normalized_schedule_expression("@reboot"),
            Some(ScheduleExpression::Reboot)
        );
        assert_eq!(
            normalized_schedule_expression("*/5 * * * *"),
            Some(ScheduleExpression::Standard("*/5 * * * *"))
        );
        assert_eq!(normalized_schedule_expression("@unknown"), None);
    }

    #[test]
    fn test_is_env_var() {
        assert!(is_env_var("SHELL=/bin/bash"));
        assert!(is_env_var("PATH=/usr/bin:/bin"));
        assert!(!is_env_var("0 0 * * * command"));
        assert!(!is_env_var("# comment"));
    }

    #[test]
    fn test_extract_cron_entry() {
        assert_eq!(
            extract_cron_entry("*/5 * * * * /usr/bin/script.sh"),
            Some((
                "*/5 * * * *".to_string(),
                "/usr/bin/script.sh".to_string(),
                None
            ))
        );
        assert_eq!(
            extract_cron_entry("0 0 * * * command with args"),
            Some((
                "0 0 * * *".to_string(),
                "command with args".to_string(),
                None
            ))
        );
        assert_eq!(
            extract_cron_entry("0 0 * * * command # note"),
            Some((
                "0 0 * * *".to_string(),
                "command".to_string(),
                Some("note".to_string())
            ))
        );
        assert_eq!(extract_cron_entry("invalid"), None);
        assert_eq!(extract_cron_entry("0 0 * *"), None);
    }
}
