use wiremock::{
    Mock, ResponseTemplate,
    matchers::{any, method, path},
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

    let response = app.post_newsletters(body).await;

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

    let response = app.post_newsletters(body).await;

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn newsletters_return_400_for_invalid_input() {
    let app = spawn_app().await;
    let test_cases = vec![
        (
            serde_json::json!({
                "content": {
                    "html": "<p>Html content</p>",
                    "text": "Text content"
                }
            }),
            "missing title",
        ),
        (
            serde_json::json!({
                "title": "Title"
            }),
            "missing content",
        ),
    ];

    for (body, error_message) in test_cases {
        let response = app.post_newsletters(body).await;

        assert_eq!(
            response.status().as_u16(),
            400,
            "The API did not fail with 400 Bad Request when the payload was {error_message}."
        );
    }
}
