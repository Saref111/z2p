use actix_web::{HttpResponse, Responder, web};
use sqlx::{PgPool, types::chrono::Utc};
use uuid::Uuid;
use tracing::Instrument;

#[derive(serde::Deserialize)]
pub struct FormData {
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    email: String,
}

pub async fn subscribe(form: web::Form<FormData>, db_pool: web::Data<PgPool>) -> impl Responder {
    let request_id = Uuid::new_v4();

    let request_span = tracing::info_span!(
        "Adding a new subscriber.",
        %request_id,
        subscriber_email = %form.email,
        subscriber_name = %form.name
    );

    let _request_span_guard = request_span.enter();

    let query_span = tracing::info_span!(
        "Saving new subscriber details in the database"
    );

    let query_result = sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(db_pool.get_ref())
    .instrument(query_span)
    .await;

    match query_result {
        Ok(_) => {
            HttpResponse::Ok().finish()
        },
        Err(err) => {
            tracing::error!("request_id {} - Failed to execute query: {:?}", request_id, err);
            HttpResponse::InternalServerError().finish()
        }
    }
}
