use std::net::TcpListener;

use z2p::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8000").unwrap();
    run(listener)?.await
}
