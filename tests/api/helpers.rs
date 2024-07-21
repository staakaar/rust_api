use once_cell::sync::Lazy;
use rust_api::configuration::{self, get_configuration, DatabaseSettings};
use rust_api::startup::{get_connection_pool, Application};
use rust_api::telemetry::{get_subscriber, init_subscriber};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use wiremock::MockServer;

static TRACING: Lazy<()> = Lazy::new(|| {
    //Logging
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub plain_text: reqwest::Url,
}

pub struct TestApp {
    pub address: String,
    pub port: u16,
    pub db_pool: PgPool,
    pub email_server: MockServer,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subdcriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub fn get_confirmation_links(&self, email_request: &wiremock::Request) -> ConfirmationLinks {
        let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

        let get_link = |s: &str| {
            let links: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .collect();
            assert_eq!(links.len(), 1);

            let raw_link = links[0].as_str().to_owned();
            let mut confirmation_link = reqwest::Url::parse(&raw_link).unwrap();
            confirmation_link.set_port(Some(self.port)).unwrap();
            confirmation_link
        };

        let html = get_link(&body["HTMLBody"].as_str().unwrap());
        let plain_text = get_link(&body["TextBody"].as_str().unwrap());

        ConfirmationLinks { html, plain_text }
    }
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    // let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    // let port = listener.local_addr().unwrap().port();
    // let address = format!("http://127.0.0.1:{}", port);

    let email_server = MockServer::start().await;

    let configuration = {
        let mut c = get_configuration().expect("Faild to read configuration");
        c.database.database_name = Uuid::new_v4().to_string();
        c.application.port = 0;
        c.email_client.base_url = email_server.uri();
        c
    };

    configure_database(&configuration.database).await;

    // let sender_email = configuration
    //     .email_client
    //     .sender()
    //     .expect("Invalid sender email address.");

    // let timeout = configuration.email_client.timeout();

    // let email_client = EmailClient::new(
    //     configuration.email_client.base_url,
    //     sender_email,
    //     configuration.email_client.authorization_token,
    //     timeout,
    // );

    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to bind address");
    let address = format!("http://127.0.0.1:{}", application.port());
    let application_port = application.port();
    let _ = tokio::spawn(application.run_until_stopped());
    TestApp {
        address,
        port: application_port,
        db_pool: get_connection_pool(&configuration.database),
        email_server,
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"CREATE DATABASE '{}';"#, config.database_name).as_ref())
        .await
        .expect("Failed to create database.");

    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}
