use actix_web::HttpResponse;

#[derive(serde::Deserialize)]
pub struct BodySchema {
    title: String,
    content: Content,
}

#[derive(serde::Deserialize)]
pub struct Content {
    text: String,
    html: String,
}

pub async fn publish_newsletter(_body: actix_web::web::Json<BodySchema>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
