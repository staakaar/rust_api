use env_logger::Env;
use rust_api::startup::Application;
use rust_api::telemetry::init_subscriber;
use rust_api::{configuration::get_configuration, telemetry::get_subscriber};
use tracing_log::LogTracer;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // Logging
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    LogTracer::init().expect("Failed to set logger");

    let subscriber = get_subscriber("rust_api".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");

    let application = Application::build(configuration).await?;
    application.run_until_stopped().await?;
    Ok(())
}
