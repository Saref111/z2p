use crate::{domain::SubscriberEmail, email_client::EmailClient};
use super::{errors::PublishError, helpers::basic_auth, types::{BodySchema, Credentials, ConfirmedSubscriber}};
use actix_web::{
    HttpRequest, HttpResponse
};
use anyhow::Context;
use secrecy::ExposeSecret;
use sha3::Digest;
use sqlx::PgPool;
use uuid::Uuid;

#[tracing::instrument(
    name = "Publish newsletter",
    skip(db_pool, email_client, body, req),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn publish_newsletter(
    body: actix_web::web::Json<BodySchema>,
    db_pool: actix_web::web::Data<PgPool>,
    email_client: actix_web::web::Data<EmailClient>,
    req: HttpRequest,
) -> Result<HttpResponse, PublishError> {
    let credentials = basic_auth(req.headers()).map_err(PublishError::AuthError)?;

    tracing::Span::current().record("username", tracing::field::display(&credentials.username));

    let user_id = validate_credentials(credentials, &db_pool).await?;

    tracing::Span::current().record("user_id", tracing::field::display(&user_id));

    let confirmed_subscribers = get_confirmed_subscribers(&db_pool).await?;

    send_newsletters(confirmed_subscribers, &email_client, &body).await?;

    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(name = "Sending newsletters to confirmed subscribers", skip_all)]
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

#[tracing::instrument(
    name = "Validating user auth credentials",
    skip(pool, credentials),
    fields(username=&credentials.username)
)]
async fn validate_credentials(
    credentials: Credentials,
    pool: &PgPool,
) -> Result<Uuid, PublishError> {
    let password_hash = sha3::Sha3_256::digest(
        credentials.password.expose_secret().as_bytes()
    );
    let password_hash = format!("{:x}", password_hash);

    let user_id = sqlx::query!(
        r#"
        SELECT user_id 
        FROM users 
        WHERE username = $1 
        AND password_hash = $2
    "#,
        credentials.username,
        password_hash
    )
    .fetch_optional(pool)
    .await
    .context("Failed to perform query to validate auth credentials.")
    .map_err(PublishError::UnexpectedError)?;

    user_id
        .map(|row| row.user_id)
        .ok_or_else(|| anyhow::anyhow!("Invalid username or password."))
        .map_err(PublishError::AuthError)
}
