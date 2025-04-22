use std::net::TcpListener;

use env_logger::Env;
use sqlx::PgPool;
use z2p::{configuration::get_configuration, startup::run};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let config = get_configuration().expect("Failed to read configuration");
    let address = format!("127.0.0.1:{}", config.app_port);
    let connection_pool = PgPool::connect(&config.database.connection_string())
        .await
        .expect("Failed to connect to Postgres");

    let listener = TcpListener::bind(address).unwrap();
    run(listener, connection_pool)?.await
}
