use z2p::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    run()?.await
}