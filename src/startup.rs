use crate::configuration::{DatabaseSettings, Settings};
use crate::email_client::EmailClient;
use crate::routes::{confirm, health_check, publish_newsletter, subscribe};
use actix_web::dev::Server;
use actix_web::{App, HttpServer, web};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use std::time::Duration;
use tracing_actix_web::TracingLogger;

pub struct Application {
    port: u16,
    server: Server,
}

pub struct ApplicationBaseURL(pub String);

impl Application {
    pub async fn build(config: Settings) -> Result<Self, std::io::Error> {
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

        let address = format!("{}:{}", config.app.host, config.app.port);
        let connection_pool = get_connection_pull(&config.database);

        let listener = TcpListener::bind(address).unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = run(listener, connection_pool, email_client, config.app.base_url)?;

        Ok(Self { port, server })
    }

    pub fn get_port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
    base_url: String,
) -> Result<Server, std::io::Error> {
    let db_pool = web::Data::new(db_pool);
    let email_client = web::Data::new(email_client);
    let base_url = web::Data::new(ApplicationBaseURL(base_url));

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .route("/subscriptions/confirm", web::get().to(confirm))
            .route("/newsletters", web::post().to(publish_newsletter))
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
            .app_data(base_url.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}

pub fn get_connection_pull(db_config: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(2))
        .connect_lazy_with(db_config.with_db())
}
