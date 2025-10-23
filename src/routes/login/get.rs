use crate::routes::helpers::prepare_html_template;
use actix_web::{HttpResponse, cookie::Cookie, http::header::ContentType};
use actix_web_flash_messages::{IncomingFlashMessages, Level};
use std::fmt::Write;

pub async fn login_form(flash_messages: IncomingFlashMessages) -> HttpResponse {
    let mut error_string = String::new();
    for m in flash_messages.iter().filter(|m| m.level() == Level::Error) {
        writeln!(error_string, "{}", m.content()).unwrap();
    }

    let page_string = prepare_html_template(&[("error", &error_string)], "login.html");

    let mut response = HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(page_string);
    response
        .add_removal_cookie(&Cookie::new("_flash", ""))
        .unwrap();
    response
}
