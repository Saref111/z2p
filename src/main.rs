use sqlx::postgres::PgPoolOptions;
use std::{net::TcpListener, time::Duration};
use z2p::{
    configuration::get_configuration,
    email_client::EmailClient,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("z2p".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let config = get_configuration().expect("Failed to read configuration");

    let email_sender = config
        .email_client
        .sender()
        .expect("Invalid sender email address.");
    let email_client = EmailClient::new(
        config.email_client.base_url,
        email_sender,
        config.email_client.auth_token,
    );

    let address = format!("{}:{}", config.app.host, config.app.port);
    let connection_pool = PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(2))
        .connect_lazy_with(config.database.with_db());

    let listener = TcpListener::bind(address).unwrap();
    run(listener, connection_pool, email_client)?.await
}
