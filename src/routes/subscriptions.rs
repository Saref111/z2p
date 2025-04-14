use actix_web::{HttpResponse, Responder, web};
use sqlx::{PgPool, types::chrono::Utc};
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    email: String,
}

pub async fn subscribe(form: web::Form<FormData>, db_pool: web::Data<PgPool>) -> impl Responder {
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
    .await;

    match query_result {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(err) => {
            println!("Failed to execute query: {}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}
