use crate::cli::actions::Action;
use anyhow::Result;
use clap::ArgMatches;
use std::path::PathBuf;

/// Convert ArgMatches into an Action
pub fn handler(matches: &ArgMatches) -> Result<Action> {
    // Determine verbosity level
    let verbose = matches.get_count("verbose") > 0;

    // Extract next count if provided
    let next = matches.get_one::<u32>("next").copied();

    // Check which mode was requested
    if matches.get_flag("crontab") {
        Ok(Action::Crontab { verbose })
    } else if let Some(file_path) = matches.get_one::<String>("file") {
        Ok(Action::File {
            path: PathBuf::from(file_path),
            verbose,
        })
    } else if let Some(expression) = matches.get_one::<String>("cron") {
        Ok(Action::Single {
            expression: expression.clone(),
            verbose,
            next,
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

    #[test]
    fn test_handler_single() {
        let matches = commands::new().get_matches_from(vec!["cron-when", "*/5 * * * *"]);
        let action = handler(&matches).unwrap();
        assert!(matches!(action, Action::Single { .. }));
    }

    #[test]
    fn test_handler_file() {
        let matches = commands::new().get_matches_from(vec!["cron-when", "-f", "test.crontab"]);
        let action = handler(&matches).unwrap();
        assert!(matches!(action, Action::File { .. }));
    }

    #[test]
    fn test_handler_crontab() {
        let matches = commands::new().get_matches_from(vec!["cron-when", "--crontab"]);
        let action = handler(&matches).unwrap();
        assert!(matches!(action, Action::Crontab { verbose: false }));
    }

    #[test]
    fn test_handler_verbose() {
        let matches = commands::new().get_matches_from(vec!["cron-when", "-v", "*/5 * * * *"]);
        let action = handler(&matches).unwrap();
        if let Action::Single { verbose, .. } = action {
            assert!(verbose);
        } else {
            panic!("Expected Single action");
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
        let action = handler(&matches).unwrap();
        if let Action::Single { next, .. } = action {
            assert_eq!(next, Some(5));
        } else {
            panic!("Expected Single action");
        }
    }
}
