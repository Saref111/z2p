use actix_web::{HttpResponse, http::header::ContentType};

use crate::{
    routes::helpers::{e500, prepare_html_template, see_other},
    session_state::TypedSession,
};

pub async fn change_password_form(session: TypedSession) -> Result<HttpResponse, actix_web::Error> {
    if session.get_user_id().map_err(e500)?.is_none() {
        return Ok(see_other("/login"));
    }

    let page = prepare_html_template(&[], "change_password_form.html");
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(page))
}
