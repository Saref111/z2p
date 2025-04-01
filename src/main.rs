use std::net::TcpListener;

use z2p::{configuration::get_configuration, startup::run};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let config = get_configuration().expect("Failed to read configuration");
    let address = format!("127.0.0.1:{}", config.app_port);
    
    let listener = TcpListener::bind(address).unwrap();
    run(listener)?.await
}
