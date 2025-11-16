use crate::routes::helpers::{get_message, prepare_html_template};
use actix_web::{HttpResponse, http::header::ContentType};
use actix_web_flash_messages::IncomingFlashMessages;

pub async fn send_newsletters_form(
    flash_messages: IncomingFlashMessages,
) -> Result<HttpResponse, actix_web::Error> {
    let idempotency_key = uuid::Uuid::new_v4();
    let message = get_message(flash_messages, None);
    let page = prepare_html_template(
        &[
            ("idempotency_key", &idempotency_key.to_string()),
            ("message", &message),
        ],
        "send_newsletters_form.html",
    );
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(page))
}
