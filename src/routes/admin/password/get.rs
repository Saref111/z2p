use actix_web::{HttpResponse, http::header::ContentType};
use actix_web_flash_messages::{IncomingFlashMessages, Level};
use std::fmt::Write;

use crate::{
    routes::helpers::{e500, prepare_html_template, see_other},
    session_state::TypedSession,
};

pub async fn change_password_form(
    session: TypedSession,
    flash_messages: IncomingFlashMessages,
) -> Result<HttpResponse, actix_web::Error> {
    if session.get_user_id().map_err(e500)?.is_none() {
        return Ok(see_other("/login"));
    }

    let mut error_string = String::new();
    for m in flash_messages.iter().filter(|m| m.level() == Level::Error) {
        writeln!(error_string, "{}", m.content()).unwrap();
    }

    let page = prepare_html_template(&[("error", &error_string)], "change_password_form.html");
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(page))
}
