use once_cell::sync::Lazy;
use reqwest::Response;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use wiremock::MockServer;
use z2p::{
    configuration::{DatabaseSettings, get_configuration},
    startup::{Application, get_connection_pull},
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
    pub email_server: MockServer,
    pub port: u16,
}

impl TestApp {
    pub async fn post_subscription(&self, body: String) -> Response {
        reqwest::Client::new()
            .post(format!("{}/subscriptions", self.address))
            .body(body)
            .header("Content-type", "application/x-www-form-urlencoded")
            .send()
            .await
            .expect("Failed to execute request.")
    }
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

    let email_server = MockServer::start().await;

    let config = {
        let mut config = get_configuration().expect("Failed to read configuration");
        config.database.database_name = Uuid::new_v4().to_string();
        config.app.port = 0;
        config.email_client.base_url = email_server.uri();
        config
    };

    configure_database(&config.database).await;

    let app = Application::build(config.clone())
        .await
        .expect("Failed to build the application");

    let port = app.get_port();
    let _ = tokio::spawn(app.run_until_stopped());

    TestApp {
        address: format!("http://127.0.0.1:{}", port),
        db_pool: get_connection_pull(&config.database),
        email_server,
        port,
    }
}
