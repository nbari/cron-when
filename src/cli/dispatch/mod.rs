use crate::cli::actions::Action;
use anyhow::Result;
use clap::ArgMatches;
use std::env;
use std::io::{IsTerminal, stdout};
use std::path::PathBuf;

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
        anyhow::bail!(
            "Please provide a cron expression, use --file, or use --crontab\n\
             Run with --help for usage information"
        )
    }
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
