use env_logger::Env;
use rust_api::startup::run;
use rust_api::telemetry::init_subscriber;
use rust_api::{configuration::get_configuration, telemetry::get_subscriber};
use sqlx::PgPool;
use std::net::TcpListener;
use tracing_log::LogTracer;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // Logging
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    LogTracer::init().expect("Failed to set logger");

    let subscriber = get_subscriber("rust_api".into(), "info".into());
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let connnection = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postges.");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;

    run(listener, connnection)?.await
}
