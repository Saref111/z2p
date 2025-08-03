use once_cell::sync::Lazy;
use reqwest::{Response, Url};
use sha3::Digest;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};
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
    pub test_user: TestUser,
}

#[derive(Debug)]
pub struct ConfirmationLinks {
    pub text: Url,
    pub html: Url,
}

impl TestApp {
    pub async fn post_newsletters(&self, body: serde_json::Value) -> Response {
        let TestUser {username, password, ..} = &self.test_user;

        reqwest::Client::new()
            .post(format!("{}/newsletters", &self.address))
            .basic_auth(username, Some(password))
            .json(&body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_subscription(&self, body: String) -> Response {
        reqwest::Client::new()
            .post(format!("{}/subscriptions", self.address))
            .body(body)
            .header("Content-type", "application/x-www-form-urlencoded")
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub fn get_confirmation_links(&self, email_request: &wiremock::Request) -> ConfirmationLinks {
        let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

        let get_link = |s: &str| {
            let links = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .collect::<Vec<_>>();

            assert_eq!(links.len(), 1);
            let raw_link = links[0].as_str().to_owned();
            let mut link = Url::parse(&raw_link).unwrap();

            link.set_port(Some(self.port)).unwrap();

            assert_eq!(link.host_str().unwrap(), "127.0.0.1");
            link
        };

        let text = get_link(body["text"].as_str().unwrap());
        let html = get_link(body["html"].as_str().unwrap());

        ConfirmationLinks { text, html }
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
    tokio::spawn(app.run_until_stopped());

    let app = TestApp {
        address: format!("http://127.0.0.1:{port}"),
        db_pool: get_connection_pull(&config.database),
        email_server,
        port,
        test_user: TestUser::generate(),
    };
    app.test_user.store(&app.db_pool).await;

    app
}

pub async fn create_unconfirmed_subscriber(app: &TestApp) -> ConfirmationLinks {
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let _mock_guard = Mock::given(path("v1/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Create unconfirmed subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;

    app.post_subscription(body.into())
        .await
        .error_for_status()
        .unwrap();

    let req = &app.email_server.received_requests().await.unwrap()[0];

    app.get_confirmation_links(req)
}

pub async fn create_confirmed_subscriber(app: &TestApp) {
    let confirmation_links = create_unconfirmed_subscriber(app).await;

    reqwest::get(confirmation_links.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
}

pub struct TestUser {
    pub user_id: Uuid,
    pub username: String,
    pub password: String,
}

impl TestUser {
    pub fn generate() -> Self {
        Self {
            user_id: Uuid::new_v4(),
            username: Uuid::new_v4().to_string(),
            password: Uuid::new_v4().to_string(),
        }
    }

    pub async fn store(&self, db_pool: &PgPool) {
        let password_hash = sha3::Sha3_256::digest(self.password.as_bytes());
        let password_hash = format!("{:x}", password_hash);

        sqlx::query!(
            r#"
            INSERT INTO users (user_id, username, password_hash) 
            VALUES ($1, $2, $3)
        "#,
            self.user_id,
            self.username,
            password_hash
        )
        .execute(db_pool)
        .await
        .expect("Failed to store test user.");
    }
}
