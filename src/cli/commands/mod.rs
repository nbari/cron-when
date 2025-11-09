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
        .color(ColorChoice::Auto)
        .styles(styles)
        .arg(
            Arg::new("cron")
                .value_name("CRON_EXPRESSION")
                .help("Cron expression (e.g., \"*/5 * * * *\")")
                .index(1),
        )
        .arg(
            Arg::new("file")
                .short('f')
                .long("file")
                .value_name("FILE")
                .help("Read from file (crontab format)"),
        )
        .arg(
            Arg::new("crontab")
                .short('l')
                .long("crontab")
                .help("Parse current user's crontab")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Show verbose output with cron expression")
                .action(ArgAction::Count),
        )
        .arg(
            Arg::new("next")
                .short('n')
                .long("next")
                .value_name("COUNT")
                .help("Show next N occurrences of the cron expression")
                .value_parser(clap::value_parser!(u32).range(1..=100)),
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
