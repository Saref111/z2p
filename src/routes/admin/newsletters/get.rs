use crate::routes::helpers::prepare_html_template;
use actix_web::{HttpResponse, http::header::ContentType};

pub async fn send_newsletters_form() -> Result<HttpResponse, actix_web::Error> {
    let page = prepare_html_template(&[], "send_newsletters_form.html");
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(page))
}
