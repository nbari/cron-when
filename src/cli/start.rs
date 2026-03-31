use crate::cli::{actions::Action, commands, dispatch, telemetry};
use anyhow::Result;

/// Map verbosity count to tracing level
const fn get_verbosity_level(verbose_count: u8) -> Option<tracing::Level> {
    match verbose_count {
        0 => None,
        1 => Some(tracing::Level::INFO),
        2 => Some(tracing::Level::DEBUG),
        _ => Some(tracing::Level::TRACE),
    }
}

/// Main entry point for the CLI - builds and returns the Action
///
/// # Errors
///
/// Returns an error if argument parsing, telemetry initialization, or action dispatch fails
pub fn start() -> Result<Action> {
    // 1. Parse command-line arguments
    let matches = commands::new().get_matches();

    // 2. Extract verbosity level
    let verbosity_level = get_verbosity_level(matches.get_count("verbose"));

    // 3. Initialize telemetry
    telemetry::init(verbosity_level)?;

    // 4. Dispatch to appropriate action
    let action = dispatch::handler(&matches)?;

    // 5. Return the action for execution by the binary
    Ok(action)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_verbosity_level() {
        assert_eq!(get_verbosity_level(0), None);
        assert_eq!(get_verbosity_level(1), Some(tracing::Level::INFO));
        assert_eq!(get_verbosity_level(2), Some(tracing::Level::DEBUG));
        assert_eq!(get_verbosity_level(3), Some(tracing::Level::TRACE));
        assert_eq!(get_verbosity_level(4), Some(tracing::Level::TRACE));
    }
}
