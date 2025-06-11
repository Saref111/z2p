use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Parameters {
    subscription_token: String,
} 

#[tracing::instrument(
    name = "Confirm a pending subscriber",
    skip(_parameters)
)]
pub async fn confirm_subscriber(_parameters: web::Query<Parameters>) -> impl Responder {
    HttpResponse::Ok().finish()
} 