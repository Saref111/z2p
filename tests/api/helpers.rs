use std::net::TcpListener;

use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use z2p::{
    configuration::{DatabaseSettings, get_configuration},
    email_client::EmailClient,
    telemetry::{get_subscriber, init_subscriber},
};

static TRACING: Lazy<()> = Lazy::new(|| {
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

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database");

    let connection_pull = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres");

    sqlx::migrate!("./migrations")
        .run(&connection_pull)
        .await
        .expect("Failed to migrate database");

    connection_pull
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let mut config = get_configuration().expect("Failed to read configuration");
    config.database.database_name = Uuid::new_v4().to_string();

    let email_sender = config
        .email_client
        .sender()
        .expect("Invalid sender email address.");

    let timeout = config.email_client.timeout();
    let email_client = EmailClient::new(
        config.email_client.base_url,
        email_sender,
        config.email_client.auth_token,
        timeout,
    );

    let connection_pool = configure_database(&config.database).await;

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port.");
    let port = listener.local_addr().unwrap().port();
    let server = z2p::startup::run(listener, connection_pool.clone(), email_client)
        .expect("Failed to bind address.");

    let _ = tokio::spawn(server);

    TestApp {
        address: format!("http://127.0.0.1:{port}"),
        db_pool: connection_pool.clone(),
    }
}
