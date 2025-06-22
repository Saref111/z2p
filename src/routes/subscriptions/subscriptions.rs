use std::fmt::Debug;

use actix_web::{HttpResponse, web};
use anyhow::Context;
use sqlx::{PgPool, Postgres, Transaction, types::chrono::Utc};
use uuid::Uuid;

use crate::{
    domain::{ConfirmationToken, NewSubscriber, SubscriberEmail},
    email_client::EmailClient,
    startup::ApplicationBaseURL,
};

use super::{
        errors::{StoreTokenError, SubscribeError},
        helpers::{get_email_html, get_email_text},
};

#[derive(serde::Deserialize)]
pub struct FormData {
    pub name: String,
    pub email: String,
}

#[tracing::instrument(
    name = "Adding a new subscriber.",
    skip(form, db_pool, email_client, base_url),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    db_pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    base_url: web::Data<ApplicationBaseURL>,
) -> Result<HttpResponse, SubscribeError> {
    let new_subscriber: NewSubscriber =
        form.0.try_into().map_err(SubscribeError::ValidationError)?;

    let existing_subscriber = try_find_subscriber_by_email(&db_pool, &new_subscriber.email)
        .await
        .context("Failed to read data from database.")?;

    if let Some((id, status)) = existing_subscriber {
        if status != "pending_confirmation" {
            return Ok(HttpResponse::Conflict().finish());
        }

        let token_string = get_stored_confirmation_token(&db_pool, id, &new_subscriber.email)
            .await
            .context("Failed to read data from database.")?;

        let confirmation_token =
            ConfirmationToken::parse(token_string).map_err(SubscribeError::ValidationError)?;

        send_email(
            email_client,
            &new_subscriber,
            base_url,
            confirmation_token.as_ref(),
        )
        .await
        .context("Failed to send a confirmation email.")?;

        return Ok(HttpResponse::Ok().finish());
    }

    let mut transaction = db_pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool.")?;

    let subscriber_id = insert_subscriber(&new_subscriber, &mut transaction)
        .await
        .context("Failed to insert a new subscriber in the database.")?;

    let confirmation_token = ConfirmationToken::new();
    store_token(&mut transaction, subscriber_id, confirmation_token.as_ref())
        .await
        .context("Failed to store the confirmation token for a new subscriber.")?;

    send_email(
        email_client,
        &new_subscriber,
        base_url,
        confirmation_token.as_ref(),
    )
    .await
    .context("Failed to send a confirmation email.")?;

    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to store a new subscriber.")?;
    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(
    name = "Sending a confirmation email to a new subscriber",
    skip(email_client, subscriber, base_url)
)]
pub async fn send_email(
    email_client: web::Data<EmailClient>,
    subscriber: &NewSubscriber,
    base_url: web::Data<ApplicationBaseURL>,
    confirmation_token: &str,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        base_url.0, confirmation_token
    );

    email_client
        .send_email(
            subscriber.email.to_owned(),
            "HELLO!".into(),
            &get_email_html(subscriber.name.as_ref(), &confirmation_link),
            &get_email_text(subscriber.name.as_ref(), &confirmation_link),
        )
        .await
}

#[tracing::instrument(name = "Trying to find existing subscriber by email")]
async fn try_find_subscriber_by_email(
    pool: &PgPool,
    email: &SubscriberEmail,
) -> Result<Option<(Uuid, String)>, sqlx::Error> {
    let result = sqlx::query!(
        r#"
            SELECT id, status FROM subscriptions WHERE email = $1
        "#,
        email.as_ref()
    )
    .fetch_optional(pool)
    .await?;

    Ok(result.map(|r| (r.id, r.status)))
}

#[tracing::instrument(name = "Getting confirmation token")]
async fn get_stored_confirmation_token(
    pool: &PgPool,
    subscriber_id: Uuid,
    subscriber_email: &SubscriberEmail,
) -> Result<String, sqlx::Error> {
    let record = sqlx::query!(
        r#"
        SELECT subscription_token FROM subscription_tokens WHERE subscriber_id = $1
    "#,
        subscriber_id
    )
    .fetch_one(pool)
    .await?;

    Ok(record.subscription_token)
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, transaction)
)]
pub async fn insert_subscriber(
    new_subscriber: &NewSubscriber,
    transaction: &mut Transaction<'_, Postgres>,
) -> Result<Uuid, sqlx::Error> {
    let id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'pending_confirmation')
        "#,
        id,
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(&mut **transaction)
    .await?;

    Ok(id)
}

#[tracing::instrument(name = "Saving new confirmation token", skip(transaction))]
async fn store_token(
    transaction: &mut Transaction<'_, Postgres>,
    id: Uuid,
    token: &str,
) -> Result<(), StoreTokenError> {
    sqlx::query!(
        r#"
        INSERT INTO subscription_tokens (subscriber_id, subscription_token) 
        VALUES ($1, $2)
        "#,
        id,
        token
    )
    .execute(&mut **transaction)
    .await
    .map_err(StoreTokenError)?;

    Ok(())
}
