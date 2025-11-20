use std::time::Duration;

use wiremock::{
    Mock, ResponseTemplate,
    matchers::{any, method, path},
};

use crate::helpers::assert_is_redirect_to;

use super::helpers::{create_confirmed_subscriber, create_unconfirmed_subscriber, spawn_app};

#[tokio::test]
async fn newsletter_creation_is_idempotent() {
    let app = spawn_app().await;
    app.login_test_user().await;
    create_confirmed_subscriber(&app).await;

    Mock::given(path("v1/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "text": "Newsletter body as plain text",
        "html": "<p>Newsletter body as HTML</p>",
        "idempotency_key": uuid::Uuid::new_v4().to_string()
    });

    let resp = app.post_newsletters(&newsletter_request_body).await;
    assert_is_redirect_to(&resp, "/admin/newsletters");

    let page_html = app.get_send_newsletters_html().await;
    assert!(page_html.contains("The newsletter issue has been published!"));

    let resp = app.post_newsletters(&newsletter_request_body).await;
    assert_is_redirect_to(&resp, "/admin/newsletters");

    let page_html = app.get_send_newsletters_html().await;
    assert!(page_html.contains("The newsletter issue has been published!"))
}

#[tokio::test]
async fn you_must_be_logged_in_to_send_newsletters() {
    let app = spawn_app().await;

    let body = serde_json::json!(
    {
        "title": "Newsletter title",
        "html": "HTML content",
        "text": "text content",
        "idempotency_key": uuid::Uuid::new_v4().to_string()
    });

    let resp = app.post_newsletters(&body).await;

    assert_is_redirect_to(&resp, "/login");
}

#[tokio::test]
async fn get_message_on_successful_newsletter_publish() {
    let app = spawn_app().await;
    app.login_test_user().await;
    create_confirmed_subscriber(&app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let body = serde_json::json!(
    {
        "title": "Newsletter title",
        "html": "HTML content",
        "text": "text content",
        "idempotency_key": uuid::Uuid::new_v4().to_string()
    });

    let resp = app.post_newsletters(&body).await;

    assert_is_redirect_to(&resp, "/admin/newsletters");

    let page_html = app.get_send_newsletters_html().await;

    assert!(page_html.contains("The newsletter issue has been published!"))
}

#[tokio::test]
async fn logged_user_reaches_send_newsletters_page() {
    let app = spawn_app().await;
    app.login_test_user().await;
    create_confirmed_subscriber(&app).await;

    let page_html = app.get_send_newsletters_html().await;

    assert!(page_html.contains("form name=\"send-newsletters\""));
}

#[tokio::test]
async fn unconfirmed_subscriber_should_not_get_a_newsletter() {
    let app = spawn_app().await;
    app.login_test_user().await;
    create_unconfirmed_subscriber(&app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.email_server)
        .await;

    let body = serde_json::json!(
    {
        "title": "Newsletter title",
        "html": "HTML content",
        "text": "text content",
        "idempotency_key": uuid::Uuid::new_v4().to_string()
    });

    let response = app.post_newsletters(&body).await;

    assert_is_redirect_to(&response, "/admin/newsletters");
}

#[tokio::test]
async fn confirmed_subscriber_should_get_a_newsletter() {
    let app = spawn_app().await;

    app.login_test_user().await;
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
        "html": "HTML content",
        "text": "text content",
        "idempotency_key": uuid::Uuid::new_v4().to_string()
    });

    let response = app.post_newsletters(&body).await;

    assert_is_redirect_to(&response, "/admin/newsletters");
}

#[tokio::test]
async fn newsletters_return_400_for_invalid_input() {
    let app = spawn_app().await;
    app.login_test_user().await;

    let test_cases = vec![
        (
            serde_json::json!({
                "html": "<p>Html content</p>",
                "text": "Text content",
                "idempotency_key": uuid::Uuid::new_v4().to_string()
            }),
            "missing title",
        ),
        (
            serde_json::json!({
                "title": "Title",
                "idempotency_key": uuid::Uuid::new_v4().to_string()
            }),
            "missing content",
        ),
        (
            serde_json::json!({
                "title": "Title",
                "html": "<p>Html content</p>",
                "text": "Text content"
            }),
            "missing idempotent key",
        ),
    ];

    for (body, error_message) in test_cases {
        let response = app.post_newsletters(&body).await;

        assert_eq!(
            response.status().as_u16(),
            400,
            "The API did not fail with 400 Bad Request when the payload was {error_message}."
        );
    }
}

#[tokio::test]
async fn concurrent_form_submission_is_handled_gracefully() {
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;
    app.login_test_user().await;

    Mock::given(path("v1/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(2)))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "text": "Newsletter body as plain text",
        "html": "<p>Newsletter body as HTML</p>",
        "idempotency_key": uuid::Uuid::new_v4().to_string()
    });

    let response1 = app.post_newsletters(&newsletter_request_body);
    let response2 = app.post_newsletters(&newsletter_request_body);
    let (response1, response2) = tokio::join!(response1, response2);
    assert_eq!(response1.status(), response2.status());
    assert_eq!(
        response1.text().await.unwrap(),
        response2.text().await.unwrap()
    );
}
