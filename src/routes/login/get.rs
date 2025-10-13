use actix_web::{HttpResponse, cookie::Cookie, http::header::ContentType};
use actix_web_flash_messages::{IncomingFlashMessages, Level};
use std::fmt::Write;
use tera::{self, Context as TeraContext};

pub async fn login_form(flash_messages: IncomingFlashMessages) -> HttpResponse {
    let mut error_string = String::new();
    for m in flash_messages.iter().filter(|m| m.level() == Level::Error) {
        writeln!(error_string, "{}", m.content()).unwrap();
    }

    let mut ctx = TeraContext::new();
    ctx.insert("error", &error_string);
    let tera = tera::Tera::new("views/**/*").expect("Failed to initialize Tera templates");
    let page_string = tera
        .render("login.html", &ctx)
        .expect("Failed rendering login page.");

    let mut response = HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(page_string);
    response
        .add_removal_cookie(&Cookie::new("_flash", ""))
        .unwrap();
    response
}
