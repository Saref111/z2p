use actix_web::{HttpResponse, http::header::ContentType};
use actix_web_flash_messages::{IncomingFlashMessages, Level};

use crate::{
    routes::helpers::{e500, get_message, prepare_html_template, see_other},
    session_state::TypedSession,
};

pub async fn change_password_form(
    session: TypedSession,
    flash_messages: IncomingFlashMessages,
) -> Result<HttpResponse, actix_web::Error> {
    if session.get_user_id().map_err(e500)?.is_none() {
        return Ok(see_other("/login"));
    }

    let error_string = get_message(flash_messages, Some(Level::Error));

    let page = prepare_html_template(&[("error", &error_string)], "change_password_form.html");
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(page))
}
