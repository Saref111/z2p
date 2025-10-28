use crate::routes::helpers::{get_message, prepare_html_template};
use actix_web::{HttpResponse, cookie::Cookie, http::header::ContentType};
use actix_web_flash_messages::IncomingFlashMessages;

pub async fn login_form(flash_messages: IncomingFlashMessages) -> HttpResponse {
    let message_string = get_message(flash_messages, None);
    let page_string = prepare_html_template(&[("message", &message_string)], "login.html");

    let mut response = HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(page_string);
    response
        .add_removal_cookie(&Cookie::new("_flash", ""))
        .unwrap();
    response
}
