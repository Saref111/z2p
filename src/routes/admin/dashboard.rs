use crate::{authentication::UserId, routes::helpers::e500, session_state::TypedSession};

use super::super::helpers::prepare_html_template;
use actix_web::{HttpResponse, http::header::ContentType, web};
use anyhow::Context;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn admin_dashboard(
    pool: web::Data<PgPool>,
    user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, actix_web::Error> {
    let username = get_username(**user_id, &pool).await.map_err(e500)?;

    let page_string = prepare_html_template(&[("username", &username)], "dashboard.html");
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(page_string))
}

#[tracing::instrument(name = "Get username", skip(pool))]
pub async fn get_username(user_id: Uuid, pool: &PgPool) -> Result<String, anyhow::Error> {
    let row = sqlx::query!(
        r#"
SELECT username
FROM users
WHERE user_id = $1
"#,
        user_id,
    )
    .fetch_one(pool)
    .await
    .context("Failed to perform a query to retrieve a username.")?;
    Ok(row.username)
}
