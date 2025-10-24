use actix_web::{HttpResponse, http::header::ContentType};

use crate::routes::helpers::prepare_html_template;

pub async fn change_password_form() -> Result<HttpResponse, actix_web::Error> {
    let page = prepare_html_template(&[], "change_password_form");
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(page))
}
