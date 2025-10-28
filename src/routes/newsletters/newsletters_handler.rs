use super::{
    errors::PublishError,
    helpers::basic_auth,
    types::{BodySchema, ConfirmedSubscriber},
};
use crate::{
    authentication::{AuthError, validate_credentials},
    domain::SubscriberEmail,
    email_client::EmailClient,
};
use actix_web::{HttpRequest, HttpResponse};
use anyhow::Context;
use sqlx::PgPool;

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

    let user_id = validate_credentials(credentials, &db_pool)
        .await
        .map_err(|e| match e {
            AuthError::InvalidCredentials(_) => PublishError::AuthError(e.into()),
            AuthError::UnexpectedError(_) => PublishError::UnexpectedError(e.into()),
        })?;

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
