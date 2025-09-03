use actix_web::{HttpResponse, http::header::ContentType, web};
use tera::{self, Context as TeraContext};

#[derive(serde::Deserialize)]
pub struct QueryParams {
    error: Option<String>,
}

pub async fn login_form(query: web::Query<QueryParams>) -> HttpResponse {
    let error = match query.0.error {
        Some(e) => e,
        None => "".into(),
    };

    let mut ctx = TeraContext::new();
    ctx.insert("error", &error);
    let tera = tera::Tera::new("views/**/*").expect("Failed to initialize Tera templates");
    let page_string = tera
        .render("login.html", &ctx)
        .expect("Failed rendering login page.");

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(page_string)
}
