use actix_web::{HttpResponse, Responder, web};

#[derive(serde::Deserialize)]
pub struct FormData {
    #[allow(dead_code)]
    username: String,
    #[allow(dead_code)]
    email: String,
}

pub async fn subscribe(_form: web::Form<FormData>) -> impl Responder {
    HttpResponse::Ok().finish()
}
