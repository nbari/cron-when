use anyhow::Result;
use cron_when::cli;
use tracing::info;

fn main() -> Result<()> {
    // Build the action from CLI arguments
    let action = cli::start()?;

    info!("Starting cron-when execution");

    // Execute the action
    match action {
        cli::actions::Action::Single {
            expression,
            verbose,
            next,
        } => {
            info!(action = "single", "Executing single expression");
            cli::actions::single::execute(&expression, verbose, next)?
        }

        cli::actions::Action::File { path, verbose } => {
            info!(action = "file", path = %path.display(), "Executing file parsing");
            cli::actions::file::execute(&path, verbose)?
        }

        cli::actions::Action::Crontab { verbose } => {
            info!(action = "crontab", "Executing crontab parsing");
            cli::actions::crontab::execute(verbose)?
        }
    }

    info!("Execution completed successfully");

    // Gracefully shutdown tracer provider if initialized
    cli::telemetry::shutdown_tracer();

    Ok(())
}
