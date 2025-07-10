use crate::{domain::SubscriberEmail, email_client::EmailClient};

use super::helpers::error_chain_fmt;
use actix_web::{
    http::{header::{self, HeaderMap, HeaderValue}, StatusCode}, HttpRequest, HttpResponse, ResponseError
};
use anyhow::Context;
use base64::Engine;
use secrecy::SecretString;
use sqlx::PgPool;

#[derive(serde::Deserialize)]
pub struct BodySchema {
    title: String,
    content: Content,
}

#[derive(serde::Deserialize)]
pub struct Content {
    text: String,
    html: String,
}

pub struct ConfirmedSubscriber {
    email: SubscriberEmail,
}

#[derive(thiserror::Error)]
pub enum PublishError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
    #[error("Authentication error.")]
    AuthError(#[source] anyhow::Error),
}

impl std::fmt::Debug for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for PublishError {
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        match self {
            PublishError::UnexpectedError(_) => HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR),
            PublishError::AuthError(_) => {
                let mut resp = HttpResponse::new(StatusCode::UNAUTHORIZED);

                let header_value = HeaderValue::from_str(r#"Basic realm="publish""#).unwrap();

                resp.headers_mut().insert(header::WWW_AUTHENTICATE, header_value);

                resp
            },
        }
    }
}

#[tracing::instrument(
    name = "Sending newsletters to confirmed subscribers",
    skip(db_pool, email_client, body)
)]
pub async fn publish_newsletter(
    body: actix_web::web::Json<BodySchema>,
    db_pool: actix_web::web::Data<PgPool>,
    email_client: actix_web::web::Data<EmailClient>,
    req: HttpRequest,
) -> Result<HttpResponse, PublishError> {
    let _creds = basic_auth(req.headers()).map_err(PublishError::AuthError)?;

    let confirmed_subscribers = get_confirmed_subscribers(&db_pool).await?;

    send_newsletters(confirmed_subscribers, &email_client, &body).await?;

    Ok(HttpResponse::Ok().finish())
}

async fn send_newsletters(
    subscribers: Vec<Result<ConfirmedSubscriber, anyhow::Error>>,
    email_client: &EmailClient,
    body: &BodySchema,
) -> Result<(), anyhow::Error> {
    let chunks = subscribers
        .iter()
        .filter_map(|s| match s {
            Ok(s) => Some(&s.email),
            Err(err) => {
                tracing::warn!(
                    err.cause_chain = ?err,
                    "Skipping the confirmed subscriber. \
                    The stored contact details are invalid."
                );
                None
            }
        })
        .collect::<Vec<&SubscriberEmail>>();

    for subscribers_chunk in chunks.chunks(50) {
        email_client
            .send_email(
                subscribers_chunk.to_vec(),
                &body.title,
                &body.content.html,
                &body.content.text,
            )
            .await
            .with_context(|| {
                format!(
                    "Failed to send newsletter issue to {:#?}",
                    &subscribers_chunk
                )
            })?;
    }

    Ok(())
}

#[tracing::instrument(name = "Get confirmed subscribers", skip(pool))]
async fn get_confirmed_subscribers(
    pool: &PgPool,
) -> Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT email FROM subscriptions WHERE status = 'confirmed';
        "#
    )
    .fetch_all(pool)
    .await?;

    let confirmed_subscribers = rows
        .into_iter()
        .map(|r| match SubscriberEmail::parse(r.email) {
            Ok(email) => Ok(ConfirmedSubscriber { email }),
            Err(err) => Err(anyhow::anyhow!(err)),
        })
        .collect();

    Ok(confirmed_subscribers)
}

struct Credentials {
    username: String,
    password: SecretString,
}

fn basic_auth(headers: &HeaderMap) -> Result<Credentials, anyhow::Error> {
    let header_value = headers
        .get("Authorization")
        .context("The 'Authorization' header is missing.")?
        .to_str()
        .context("The 'Authorization' header was not a valid UTF8 string.")?;

    let base64_encoded_segment = header_value
        .strip_prefix("Basic")
        .context("The authorization scheme is not 'Basic'")?;

    let decoded_bytes = base64::engine::general_purpose::STANDARD
        .decode(base64_encoded_segment)
        .context("Failed to base64-decode 'Basic' credentials")?;

    let decoded_creds =
        String::from_utf8(decoded_bytes).context("The decode credential string is not UTF8")?;

    let mut creds = decoded_creds.splitn(2, ":");
    let username = creds
        .next()
        .ok_or_else(|| anyhow::anyhow!("A username must be provided in 'Basic' auth."))?
        .to_string();
    let password = creds
        .next()
        .ok_or_else(|| anyhow::anyhow!("A password must be provided in 'Basic' auth."))?
        .to_string();

    Ok(Credentials {
        username,
        password: SecretString::from(password),
    })
}
