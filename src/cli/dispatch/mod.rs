use crate::cli::actions::Action;
use crate::crontab;
use anyhow::Result;
use clap::ArgMatches;
use colored::Colorize;
use std::env;
use std::fmt;
use std::io::{IsTerminal, stdout};
use std::path::{Path, PathBuf};

/// Convert `ArgMatches` into an Action
///
/// # Errors
///
/// Returns an error if no valid action can be determined from the matches
pub fn handler(matches: &ArgMatches) -> Result<Action> {
    // Determine verbosity level
    let verbose = matches.get_count("verbose") > 0;

    // Extract next count if provided
    let next = matches.get_one::<u32>("next").copied();

    // Determine if color should be enabled based on hierarchy:
    // 1. --no-color flag (highest priority)
    // 2. --color flag
    // 3. NO_COLOR environment variable (https://no-color.org/)
    // 4. CLICOLOR_FORCE environment variable
    // 5. stdout is a terminal (auto-detection)
    let color = if matches.get_flag("no-color") {
        false
    } else if matches.get_flag("color") {
        true
    } else if env::var_os("NO_COLOR").is_some() {
        false
    } else if env::var_os("CLICOLOR_FORCE").is_some_and(|v| v != "0") {
        true
    } else {
        stdout().is_terminal()
    };

    // Check which mode was requested
    if matches.get_flag("crontab") {
        Ok(Action::Crontab { verbose, color })
    } else if let Some(file_path) = matches.get_one::<String>("file") {
        Ok(Action::File {
            path: PathBuf::from(file_path),
            verbose,
            color,
        })
    } else if let Some(expression) = matches.get_one::<String>("cron") {
        Ok(Action::Single {
            expression: expression.clone(),
            verbose,
            next,
            color,
        })
    } else {
        Err(UsageHint::new(etc_crontab_entries(), user_crontab_entries(), color).into())
    }
}

/// Error returned when the CLI is invoked without an action; carries a
/// pre-rendered, color-aware usage hint for the user
#[derive(Debug)]
pub struct UsageHint {
    rendered: String,
}

impl UsageHint {
    /// Build a hint from best-effort entry counts of the system and user crontabs
    fn new(etc_entries: Option<usize>, user_entries: Option<usize>, color: bool) -> Self {
        Self {
            rendered: build_usage_hint(etc_entries, user_entries, color),
        }
    }
}

impl fmt::Display for UsageHint {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.rendered)
    }
}

impl std::error::Error for UsageHint {}

const BASE_HINT: &str = "Please provide a cron expression, use --file or use --crontab";
const HELP_HINT: &str = "Run with --help for usage information";
const ETC_CRONTAB_PATH: &str = "/etc/crontab";

/// Compose the usage hint message shown when no action is requested
fn build_usage_hint(
    etc_entries: Option<usize>,
    user_entries: Option<usize>,
    color: bool,
) -> String {
    if color {
        colored::control::set_override(true);
    }

    let base = if color {
        BASE_HINT.yellow().bold().to_string()
    } else {
        BASE_HINT.to_string()
    };

    let mut lines = vec![base];

    if let Some(count) = etc_entries {
        lines.push(format!(
            "In {ETC_CRONTAB_PATH} you have {}",
            entry_count_label(count)
        ));
    }

    if let Some(count) = user_entries {
        lines.push(format!(
            "In your current crontab you have {}",
            entry_count_label(count)
        ));
    }

    if color {
        lines.push(HELP_HINT.bright_black().to_string());
    } else {
        lines.push(HELP_HINT.to_string());
    }

    lines.join("\n")
}

/// Render an entry count with correct pluralization
fn entry_count_label(count: usize) -> String {
    if count == 1 {
        "1 entry".to_string()
    } else {
        format!("{count} entries")
    }
}

/// Count entries in the system crontab, `None` when it is not readable
fn etc_crontab_entries() -> Option<usize> {
    crontab::parse_file(Path::new(ETC_CRONTAB_PATH))
        .ok()
        .map(|entries| entries.len())
}

/// Count entries in the current user's crontab, `None` when it cannot be listed
fn user_crontab_entries() -> Option<usize> {
    crontab::parse_current().ok().map(|entries| entries.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::commands;
    use std::sync::{Mutex, MutexGuard};

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn env_guard() -> MutexGuard<'static, ()> {
        ENV_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    #[test]
    fn test_handler_single() {
        let matches = commands::new().get_matches_from(vec!["cron-when", "*/5 * * * *"]);
        let action = handler(&matches);
        assert!(action.is_ok());
        if let Ok(action) = action {
            assert!(matches!(action, Action::Single { .. }));
        }
    }

    #[test]
    fn test_handler_file() {
        let matches = commands::new().get_matches_from(vec!["cron-when", "-f", "test.crontab"]);
        let action = handler(&matches);
        assert!(action.is_ok());
        if let Ok(action) = action {
            assert!(matches!(action, Action::File { .. }));
        }
    }

    #[test]
    fn test_handler_crontab() {
        let matches = commands::new().get_matches_from(vec!["cron-when", "--crontab"]);
        let action = handler(&matches);
        assert!(action.is_ok());
        if let Ok(action) = action {
            assert!(matches!(action, Action::Crontab { verbose: false, .. }));
        }
    }

    #[test]
    fn test_handler_verbose() {
        let matches = commands::new().get_matches_from(vec!["cron-when", "-v", "*/5 * * * *"]);
        let action = handler(&matches);
        assert!(action.is_ok());
        if let Ok(Action::Single { verbose, .. }) = action {
            assert!(verbose);
        }
    }

    #[test]
    fn test_handler_no_args() {
        let matches = commands::new().get_matches_from(vec!["cron-when"]);
        let result = handler(&matches);
        assert!(result.is_err());
        if let Err(error) = result {
            let hint = error.downcast_ref::<UsageHint>();
            assert!(hint.is_some());
            if let Some(hint) = hint {
                let message = hint.to_string();
                assert!(message.contains(BASE_HINT));
                assert!(message.contains(HELP_HINT));
            }
        }
    }

    #[test]
    fn test_build_usage_hint_with_counts() {
        let hint = build_usage_hint(Some(3), Some(1), false);
        assert_eq!(
            hint,
            "Please provide a cron expression, use --file or use --crontab\n\
             In /etc/crontab you have 3 entries\n\
             In your current crontab you have 1 entry\n\
             Run with --help for usage information"
        );
    }

    #[test]
    fn test_build_usage_hint_omits_unavailable_counts() {
        let hint = build_usage_hint(None, None, false);
        assert_eq!(
            hint,
            "Please provide a cron expression, use --file or use --crontab\n\
             Run with --help for usage information"
        );

        let empty_etc = build_usage_hint(Some(0), None, false);
        assert!(empty_etc.contains("In /etc/crontab you have 0 entries"));
        assert!(!empty_etc.contains("current crontab"));
    }

    #[test]
    fn test_build_usage_hint_color() {
        let styled = build_usage_hint(None, None, true);
        assert!(styled.contains("\u{1b}["));
        colored::control::unset_override();

        let plain = build_usage_hint(None, None, false);
        assert!(!plain.contains("\u{1b}["));
    }

    #[test]
    fn test_handler_next() {
        let matches =
            commands::new().get_matches_from(vec!["cron-when", "--next", "5", "*/5 * * * *"]);
        let action = handler(&matches);
        assert!(action.is_ok());
        if let Ok(Action::Single { next, .. }) = action {
            assert_eq!(next, Some(5));
        }
    }

    #[test]
    #[allow(clippy::undocumented_unsafe_blocks)]
    fn test_handler_no_color_env() {
        let _guard = env_guard();
        unsafe {
            env::set_var("NO_COLOR", "1");
        }
        let matches = commands::new().get_matches_from(vec!["cron-when", "*/5 * * * *"]);
        let action = handler(&matches);
        assert!(action.is_ok());
        if let Ok(Action::Single { color, .. }) = action {
            // Should be false because of NO_COLOR
            assert!(!color);
        }
        unsafe {
            env::remove_var("NO_COLOR");
        }
    }

    #[test]
    #[allow(clippy::undocumented_unsafe_blocks)]
    fn test_handler_empty_no_color_env_disables_color() {
        let _guard = env_guard();
        unsafe {
            env::set_var("NO_COLOR", "");
            env::set_var("CLICOLOR_FORCE", "1");
        }
        let matches = commands::new().get_matches_from(vec!["cron-when", "*/5 * * * *"]);
        let action = handler(&matches);
        assert!(action.is_ok());
        if let Ok(Action::Single { color, .. }) = action {
            assert!(!color);
        }
        unsafe {
            env::remove_var("NO_COLOR");
            env::remove_var("CLICOLOR_FORCE");
        }
    }

    #[test]
    #[allow(clippy::undocumented_unsafe_blocks)]
    fn test_handler_color_flag_overrides_no_color_env() {
        let _guard = env_guard();
        unsafe {
            env::set_var("NO_COLOR", "1");
        }
        let matches = commands::new().get_matches_from(vec!["cron-when", "--color", "*/5 * * * *"]);
        let action = handler(&matches);
        assert!(action.is_ok());
        if let Ok(Action::Single { color, .. }) = action {
            // Flag should override environment variable
            assert!(color);
        }
        unsafe {
            env::remove_var("NO_COLOR");
        }
    }

    #[test]
    #[allow(clippy::undocumented_unsafe_blocks)]
    fn test_handler_no_color_flag_overrides_all() {
        let _guard = env_guard();
        unsafe {
            env::set_var("CLICOLOR_FORCE", "1");
        }
        let matches =
            commands::new().get_matches_from(vec!["cron-when", "--no-color", "*/5 * * * *"]);
        let action = handler(&matches);
        assert!(action.is_ok());
        if let Ok(Action::Single { color, .. }) = action {
            // --no-color flag should override CLICOLOR_FORCE
            assert!(!color);
        }
        unsafe {
            env::remove_var("CLICOLOR_FORCE");
        }
    }
}
