use std::fmt::{Debug, Display};

use env_logger::Env;
use rust_api::issue_delivery_worker::run_worker_until_stopped;
use rust_api::startup::Application;
use rust_api::telemetry::init_subscriber;
use rust_api::{configuration::get_configuration, telemetry::get_subscriber};
use tokio::task::JoinError;
use tracing_log::LogTracer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Logging
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    LogTracer::init().expect("Failed to set logger");

    let subscriber = get_subscriber("rust_api".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");

    let application = Application::build(configuration).await?;
    let application_task = tokio::spawn(application.run_until_stopped());
    let worker_task = tokio::spawn(run_worker_until_stopped(configuration));

    tokio::select! {
        o = application_task => report_exit("API", o),
        o = worker_task => report_exit("Background worker", o),
    };
    Ok(())
}

fn export_exit(task_name: &str, outcome: Result<Result<(), impl Debug + Display>, JoinError>) {
    match outcome {
        Ok(Ok(())) => {
            tracing::info!("{} has exited", task_name)
        }
        Ok(Err(e)) => {
            tracing::error!(error.cause_chain = ?e, error.message = %e, "{} failed", task_name)
        }
        Err(e) => {
            tracing::error!(error.cause_chain = ?e, error.message = %e, "{}' task failed to complete", task_name)
        }
    }
}
