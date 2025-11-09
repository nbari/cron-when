use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;
use tracing::{debug, info, instrument, warn};

#[derive(Debug, Clone)]
pub struct CronEntry {
    pub expression: String,
    pub comment: Option<String>,
}

/// Parse crontab from current user
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
        anyhow::bail!("crontab -l failed: {}", stderr);
    }

    let content =
        String::from_utf8(output.stdout).context("Failed to parse crontab output as UTF-8")?;

    let entries = parse_content(&content);
    info!(entry_count = entries.len(), "Parsed crontab entries");

    Ok(entries)
}

/// Parse crontab from file
#[instrument(level = "info", fields(path = %path.display()))]
pub fn parse_file(path: &Path) -> Result<Vec<CronEntry>> {
    debug!("Reading crontab file");

    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let entries = parse_content(&content);
    info!(entry_count = entries.len(), "Parsed crontab file entries");

    Ok(entries)
}

/// Parse crontab content and extract entries
#[instrument(level = "debug", skip(content))]
fn parse_content(content: &str) -> Vec<CronEntry> {
    let mut entries = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    debug!(line_count = lines.len(), "Parsing crontab content");

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Skip environment variable assignments
        if is_env_var(trimmed) {
            debug!(line = trimmed, "Skipping environment variable");
            continue;
        }

        // Try to parse as cron entry
        if let Some(expression) = extract_cron_expression(trimmed) {
            // Check if previous line was a comment
            let comment = extract_previous_comment(&lines, i);
            debug!(expression = %expression, has_comment = comment.is_some(), "Found cron entry");
            entries.push(CronEntry {
                expression,
                comment,
            });
        } else {
            warn!(line = trimmed, "Could not parse as cron expression");
        }
    }

    entries
}

/// Check if line is an environment variable assignment
fn is_env_var(line: &str) -> bool {
    if !line.contains('=') {
        return false;
    }

    let parts: Vec<&str> = line.splitn(2, '=').collect();
    if parts.len() == 2 && !parts[0].contains(char::is_whitespace) {
        return true;
    }

    false
}

/// Extract cron expression from a line (first 5 fields)
fn extract_cron_expression(line: &str) -> Option<String> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 6 {
        // Standard cron: 5 time fields + command
        Some(parts[0..5].join(" "))
    } else {
        None
    }
}

/// Extract comment from previous line if it exists
fn extract_previous_comment(lines: &[&str], current_index: usize) -> Option<String> {
    if current_index > 0 {
        let prev_line = lines[current_index - 1].trim();
        if prev_line.starts_with('#') {
            return Some(prev_line.trim_start_matches('#').trim().to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_content() {
        let content = r#"
# Run every 5 minutes
*/5 * * * * /usr/bin/script1.sh

# Daily backup at midnight
0 0 * * * /usr/bin/backup.sh

# Environment variable
SHELL=/bin/bash

# Another comment without entry

30 2 * * 1 /usr/bin/weekly.sh
"#;

        let entries = parse_content(content);
        assert_eq!(entries.len(), 3);

        assert_eq!(entries[0].expression, "*/5 * * * *");
        assert_eq!(entries[0].comment, Some("Run every 5 minutes".to_string()));

        assert_eq!(entries[1].expression, "0 0 * * *");
        assert_eq!(
            entries[1].comment,
            Some("Daily backup at midnight".to_string())
        );

        assert_eq!(entries[2].expression, "30 2 * * 1");
        assert_eq!(entries[2].comment, None);
    }

    #[test]
    fn test_parse_content_empty() {
        let content = r#"
# Just comments
# No actual entries
SHELL=/bin/bash
"#;

        let entries = parse_content(content);
        assert_eq!(entries.len(), 0);
    }

    #[test]
    fn test_is_env_var() {
        assert!(is_env_var("SHELL=/bin/bash"));
        assert!(is_env_var("PATH=/usr/bin:/bin"));
        assert!(!is_env_var("0 0 * * * command"));
        assert!(!is_env_var("# comment"));
    }

    #[test]
    fn test_extract_cron_expression() {
        assert_eq!(
            extract_cron_expression("*/5 * * * * /usr/bin/script.sh"),
            Some("*/5 * * * *".to_string())
        );
        assert_eq!(
            extract_cron_expression("0 0 * * * command with args"),
            Some("0 0 * * *".to_string())
        );
        assert_eq!(extract_cron_expression("invalid"), None);
        assert_eq!(extract_cron_expression("0 0 * *"), None);
    }
}
