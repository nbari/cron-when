use anyhow::Result;
use cron_when::cli;
use tracing::info;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    // Build the action from CLI arguments
    let action = cli::start()?;

    info!("Starting cron-when execution");

    // Execute the action
    match action {
        cli::actions::Action::Single {
            expression,
            verbose,
            next,
            color,
        } => {
            info!(action = "single", "Executing single expression");
            cli::actions::single::execute(&expression, verbose, next, color)?;
        }

        cli::actions::Action::File {
            path,
            verbose,
            color,
        } => {
            info!(action = "file", path = %path.display(), "Executing file parsing");
            cli::actions::file::execute(&path, verbose, color)?;
        }

        cli::actions::Action::Crontab { verbose, color } => {
            info!(action = "crontab", "Executing crontab parsing");
            cli::actions::crontab::execute(verbose, color)?;
        }
    }

    info!("Execution completed successfully");

    // Gracefully shutdown tracer provider if initialized
    // Note: This may timeout if OTLP endpoint is slow/unavailable,
    // but spans should still be sent
    cli::telemetry::shutdown_tracer();

    // Give a brief moment for final async operations
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    Ok(())
}
