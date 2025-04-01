mod  routes;

use std::net::TcpListener;

use actix_web::dev::Server;
use actix_web::{App, HttpRequest, HttpServer, Responder, web};
use routes::{health_check, subscribe};

async fn hello(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("world");
    format!("Hello {name}!")
}

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(hello))
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .route("/{name}", web::get().to(hello))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
