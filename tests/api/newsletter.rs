use wiremock::{
    matchers::{any, path, method}, Mock, ResponseTemplate
};

use crate::helpers::create_confirmed_subscriber;

use super::helpers::{create_unconfirmed_subscriber, spawn_app};

#[tokio::test]
async fn unconfirmed_subscriber_should_not_get_a_newsletter() {
    let app = spawn_app().await;
    create_unconfirmed_subscriber(&app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.email_server)
        .await;

    let body = serde_json::json!(
    {
        "title": "Newsletter title",
        "content": {
            "html": "HTML content",
            "text": "text content"
        }
    });

    let response = reqwest::Client::new()
        .post(format!("{}/newsletters", &app.address))
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn confirmed_subscriber_should_get_a_newsletter() {
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;

    Mock::given(path("v1/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let body = serde_json::json!(
    {
        "title": "Newsletter title",
        "content": {
            "html": "HTML content",
            "text": "text content"
        }
    });

    let response = reqwest::Client::new()
        .post(format!("{}/newsletters", &app.address))
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 200);
}
