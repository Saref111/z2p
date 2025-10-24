use actix_web::{HttpResponse, web};
use secrecy::SecretString;
use serde::Deserialize;

use crate::{
    routes::helpers::{e500, see_other},
    session_state::TypedSession,
};

#[derive(Deserialize)]
pub struct FormData {
    current_password: SecretString,
    new_password: SecretString,
    new_password_check: SecretString,
}

pub async fn change_password(
    form: web::Form<FormData>,
    session: TypedSession,
) -> Result<HttpResponse, actix_web::Error> {
    if session.get_user_id().map_err(e500)?.is_none() {
        return Ok(see_other("/login"));
    }
    todo!()
}
