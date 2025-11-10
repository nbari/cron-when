use clap::{
    Arg, ArgAction, ColorChoice, Command,
    builder::styling::{AnsiColor, Effects, Styles},
};

pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

/// Build the CLI command structure
pub fn new() -> Command {
    let styles = Styles::styled()
        .header(AnsiColor::Yellow.on_default() | Effects::BOLD)
        .usage(AnsiColor::Green.on_default() | Effects::BOLD)
        .literal(AnsiColor::Blue.on_default() | Effects::BOLD)
        .placeholder(AnsiColor::Green.on_default());

    let git_hash = built_info::GIT_COMMIT_HASH.unwrap_or("unknown");
    let long_version: &'static str =
        Box::leak(format!("{} - {}", env!("CARGO_PKG_VERSION"), git_hash).into_boxed_str());

    Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .long_version(long_version)
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .long_about(
            "A CLI tool to parse cron expressions and display next execution times with human-readable durations.\n\n\
             EXAMPLES:\n  \
             cron-when \"*/5 * * * *\"\n    \
             Show next execution time for a cron expression\n\n  \
             cron-when \"0 0 * * *\" --next 10\n    \
             Show next 10 execution times\n\n  \
             cron-when --file /etc/cron.d/jobs\n    \
             Parse and display all cron jobs from a file\n\n  \
             cron-when --crontab\n    \
             Parse current user's crontab\n\n  \
             cron-when --file /etc/cron.d/jobs --color\n    \
             Show output with colors"
        )
        .color(ColorChoice::Auto)
        .styles(styles)
        .arg(
            Arg::new("cron")
                .value_name("CRON_EXPRESSION")
                .help("Cron expression (e.g., \"*/5 * * * *\")")
                .long_help(
                    "A standard cron expression with 5 fields:\n  \
                     minute hour day month weekday\n\n\
                     Examples:\n  \
                     \"*/5 * * * *\"    - Every 5 minutes\n  \
                     \"0 * * * *\"      - Every hour\n  \
                     \"0 0 * * *\"      - Every day at midnight\n  \
                     \"0 9-17 * * 1-5\" - Weekdays 9am-5pm"
                )
                .index(1),
        )
        .arg(
            Arg::new("file")
                .short('f')
                .long("file")
                .value_name("FILE")
                .help("Read from file (crontab format)")
                .long_help(
                    "Read cron expressions from a file in standard crontab format.\n\n\
                     The file should contain lines in the format:\n  \
                     <cron-expression> <command>\n\n\
                     Lines starting with '#' are treated as comments.\n\
                     Empty lines and environment variable assignments (e.g., SHELL=/bin/bash) are ignored."
                ),
        )
        .arg(
            Arg::new("crontab")
                .short('l')
                .long("crontab")
                .help("Parse current user's crontab")
                .long_help(
                    "Parse and display all cron jobs from the current user's crontab.\n\n\
                     This is equivalent to parsing the output of 'crontab -l'.\n\
                     Requires the 'crontab' command to be available on the system."
                )
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Show verbose output with cron expression")
                .long_help(
                    "Enable verbose logging for debugging purposes.\n\n\
                     Can be specified multiple times to increase verbosity:\n  \
                     -v    = INFO level\n  \
                     -vv   = DEBUG level\n  \
                     -vvv  = TRACE level\n\n\
                     Note: Verbose output is sent to stderr via the RUST_LOG environment variable."
                )
                .action(ArgAction::Count),
        )
        .arg(
            Arg::new("next")
                .short('n')
                .long("next")
                .value_name("COUNT")
                .help("Show next N occurrences of the cron expression")
                .long_help(
                    "Show the next N execution times for a cron expression.\n\n\
                     This option displays multiple consecutive execution times with their\n\
                     corresponding delays from the current time.\n\n\
                     CONSTRAINTS:\n  \
                     COUNT must be between 1 and 100 (inclusive)\n\n\
                     EXAMPLES:\n  \
                     cron-when \"0 * * * *\" --next 5\n    \
                     Show next 5 hourly executions\n\n  \
                     cron-when \"0 0 * * 0\" -n 10\n    \
                     Show next 10 weekly executions (every Sunday)"
                )
                .value_parser(clap::value_parser!(u32).range(1..=100)),
        )
        .arg(
            Arg::new("color")
                .short('c')
                .long("color")
                .help("Enable colored output")
                .long_help(
                    "Enable colored output for better readability.\n\n\
                     When enabled, output labels and values are colored:\n  \
                     - Cron: label (Green bold), expression (Yellow)\n  \
                     - Command: label (Red bold), command (White)\n  \
                     - Next: label (Blue bold), datetime (default)\n  \
                     - Left: label (Yellow bold), duration (default)\n  \
                     - Comments: Gray\n\n\
                     This flag is useful when viewing output in a terminal that supports ANSI colors.\n\n\
                     ENVIRONMENT VARIABLE:\n  \
                     Color output can also be enabled by setting the environment variable:\n  \
                     CRON_WHEN_COLOR=1\n\n\
                     The --color flag takes precedence over the environment variable."
                )
                .action(ArgAction::SetTrue),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_structure() {
        let cmd = new();
        assert_eq!(cmd.get_name(), env!("CARGO_PKG_NAME"));
    }

    #[test]
    fn test_parse_cron_expression() {
        let matches = new().get_matches_from(vec!["cron-when", "*/5 * * * *"]);
        assert!(matches.contains_id("cron"));
        assert_eq!(matches.get_one::<String>("cron").unwrap(), "*/5 * * * *");
    }

    #[test]
    fn test_parse_file_flag() {
        let matches = new().get_matches_from(vec!["cron-when", "-f", "test.crontab"]);
        assert!(matches.contains_id("file"));
        assert_eq!(matches.get_one::<String>("file").unwrap(), "test.crontab");
    }

    #[test]
    fn test_parse_crontab_flag() {
        let matches = new().get_matches_from(vec!["cron-when", "--crontab"]);
        assert!(matches.get_flag("crontab"));
    }

    #[test]
    fn test_verbose_count() {
        let matches = new().get_matches_from(vec!["cron-when", "-vvv", "*/5 * * * *"]);
        assert_eq!(matches.get_count("verbose"), 3);
    }

    #[test]
    fn test_parse_next_flag() {
        let matches = new().get_matches_from(vec!["cron-when", "--next", "5", "*/5 * * * *"]);
        assert_eq!(matches.get_one::<u32>("next"), Some(&5));
    }

    #[test]
    fn test_parse_next_short() {
        let matches = new().get_matches_from(vec!["cron-when", "-n", "10", "*/5 * * * *"]);
        assert_eq!(matches.get_one::<u32>("next"), Some(&10));
    }
}
