use crate::{domain::SubscriberEmail, email_client::EmailClient};

use super::helpers::error_chain_fmt;
use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use anyhow::Context;
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
}

impl std::fmt::Debug for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for PublishError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            PublishError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
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
) -> Result<HttpResponse, PublishError> {
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
