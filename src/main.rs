use secrecy::ExposeSecret;
use sqlx::postgres::PgPoolOptions;
use std::{net::TcpListener, time::Duration};
use z2p::{
    configuration::get_configuration,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let config = get_configuration().expect("Failed to read configuration");
    let address = format!("{}:{}", config.app.host, config.app.port);
    let connection_pool = PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(2))
        .connect_lazy(config.database.connection_string().expose_secret())
        .expect("Failed to connect to Postgres");

    let listener = TcpListener::bind(address).unwrap();
    run(listener, connection_pool)?.await
}
