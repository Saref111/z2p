use actix_web::{HttpResponse, Responder, web};
use sqlx::{PgPool, types::chrono::Utc};
use uuid::Uuid;

use crate::{
    domain::{NewSubscriber, SubscriberEmail},
    email_client::EmailClient,
};

#[derive(serde::Deserialize)]
pub struct FormData {
    pub name: String,
    pub email: String,
}

#[tracing::instrument(
    name = "Adding a new subscriber.",
    skip(form, db_pool, email_client),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    db_pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
) -> impl Responder {
    let new_subscriber = match form.0.try_into() {
        Ok(s) => s,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    if insert_subscriber(&new_subscriber, &db_pool).await.is_err() {
        return HttpResponse::InternalServerError().finish();
    }

    if send_email(email_client, new_subscriber.email)
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().finish()
}

#[tracing::instrument(
    name = "Sending a confirmation email to a new subscriber",
    skip(email_client, email)
)]
pub async fn send_email(
    email_client: web::Data<EmailClient>,
    email: SubscriberEmail,
) -> Result<(), reqwest::Error> {
    let confirmation_link = "https://somelinktomyapi.com";

    email_client
        .send_email(
            email,
            "HELLO!".into(),
            &format!(
                "Hello new subscriber <a href=\"{}\">Click here</a>",
                confirmation_link
            ),
            &format!("Hello new subscriber Click here: {}", confirmation_link),
        )
        .await
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, pool)
)]
pub async fn insert_subscriber(
    new_subscriber: &NewSubscriber,
    pool: &web::Data<PgPool>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'pending_confirmation')
        "#,
        Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(pool.get_ref())
    .await
    .map_err(|err| {
        tracing::error!("Failed to execute query: {:?}", err);
        err
    })?;

    Ok(())
}
