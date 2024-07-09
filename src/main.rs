use env_logger::Env;
use rust_api::email_client::{self, EmailClient};
use rust_api::startup::run;
use rust_api::telemetry::init_subscriber;
use rust_api::{configuration::get_configuration, telemetry::get_subscriber};
use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use tracing_log::LogTracer;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // Logging
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    LogTracer::init().expect("Failed to set logger");

    let subscriber = get_subscriber("rust_api".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");

    // let connection_pool =
    //     PgPool::connect_lazy(&configuration.database.connection_string().expose_secret())
    //         .expect("Failed to create Postgres connection pool.");
    let connection_pool =
        PgPoolOptions::new().connect_lazy_with(configuration.database.without_db());

    // EmailClient
    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address.");
    let timeout = configuration.email_client.timeout();
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender_email,
        configuration.email_client.authorization_token,
        timeout,
    );

    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(address)?;

    run(listener, connection_pool, email_client)?.await?;
    Ok(())
}
