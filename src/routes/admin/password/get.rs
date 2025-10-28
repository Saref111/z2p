use actix_web::{HttpResponse, http::header::ContentType};
use actix_web_flash_messages::IncomingFlashMessages;

use crate::routes::helpers::{get_message, prepare_html_template};

pub async fn change_password_form(
    flash_messages: IncomingFlashMessages,
) -> Result<HttpResponse, actix_web::Error> {
    let message_string = get_message(flash_messages, None);

    let page = prepare_html_template(&[("message", &message_string)], "change_password_form.html");
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(page))
}
