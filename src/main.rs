use actix_web::{App, HttpRequest, HttpServer, Responder, web};

async fn hello(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("world");
    format!("Hello {name}!")
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(hello))
            .route("/{name}", web::get().to(hello))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
