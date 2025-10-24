use actix_web::{HttpResponse, web};
use secrecy::SecretString;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct FormData {
    current_password: SecretString,
    new_password: SecretString,
    new_password_check: SecretString,
}

pub async fn change_password(form: web::Form<FormData>) -> Result<HttpResponse, actix_web::Error> {
    todo!()
}
