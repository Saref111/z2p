use actix_web::{HttpResponse, http::header::ContentType, web};
use hmac::{Hmac, Mac};
use secrecy::ExposeSecret;
use tera::{self, Context as TeraContext};

use crate::startup::HmacSecret;

#[derive(serde::Deserialize)]
pub struct QueryParams {
    error: String,
    tag: String,
}

impl QueryParams {
    fn verify(self, secret: &HmacSecret) -> Result<String, anyhow::Error> {
        let tag = hex::decode(self.tag)?;
        let query_string = format!("error={}", urlencoding::Encoded::new(&self.error));
        let mut mac =
            Hmac::<sha2::Sha256>::new_from_slice(secret.0.expose_secret().as_bytes()).unwrap();
        mac.update(query_string.as_bytes());
        mac.verify_slice(&tag)?;
        Ok(self.error)
    }
}

pub async fn login_form(
    query: Option<web::Query<QueryParams>>,
    secret: web::Data<HmacSecret>,
) -> HttpResponse {
    let error = match query {
        Some(q) => match q.0.verify(&secret) {
            Ok(error) => {
                format!("{}", htmlescape::encode_minimal(&error))
            }
            Err(e) => {
                tracing::warn!(
                error.message = %e,
                error.cause_chain = ?e,
                "Failed to verify query parameters using the HMAC tag"
                );
                "".into()
            }
        },
        None => "".into(),
    };

    let mut ctx = TeraContext::new();
    ctx.insert("error", &error);
    let tera = tera::Tera::new("views/**/*").expect("Failed to initialize Tera templates");
    let page_string = tera
        .render("login.html", &ctx)
        .expect("Failed rendering login page.");

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(page_string)
}
