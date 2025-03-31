use actix_web::{App, HttpRequest, HttpResponse, HttpServer, Responder, web};

async fn hello(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("world");
    format!("Hello {name}!")
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(hello))
            .route("/{name}", web::get().to(hello))
            .route("/health_check", web::get().to(health_check))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
