use secrecy::ExposeSecret;
use sqlx::PgPool;
use std::net::TcpListener;
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
    let address = format!("127.0.0.1:{}", config.app_port);
    let connection_pool = PgPool::connect_lazy(
        config.database.connection_string().expose_secret()
    ).expect("Failed to connect to Postgres");

    let listener = TcpListener::bind(address).unwrap();
    run(listener, connection_pool)?.await
}
