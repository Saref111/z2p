use actix_web::{App, HttpServer, Responder, web};

async fn hello() -> impl Responder {
    "Hello world!"
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/", web::get().to(hello)))
        .bind("127.0.0.1:8000")?
        .run()
        .await
}
