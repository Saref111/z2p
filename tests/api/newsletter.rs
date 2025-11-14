use uuid::Uuid;
use wiremock::{
    Mock, ResponseTemplate,
    matchers::{any, method, path},
};

use super::helpers::{create_confirmed_subscriber, create_unconfirmed_subscriber, spawn_app};

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
    app.login_test_user().await;

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

#[tokio::test]
async fn request_missing_auth_rejected() {
    let app = spawn_app().await;

    let resp = reqwest::Client::new()
        .post(&format!("{}/newsletters", app.address))
        .json(&serde_json::json!({
            "title": "Some title",
            "content": {
                "html": "<p>HTML letter</p>",
                "text": "Text letter"
            }
        }))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(resp.status().as_u16(), 401);
    assert_eq!(
        r#"Basic realm="publish""#,
        resp.headers()["WWW-Authenticate"]
    );
}

#[tokio::test]
async fn non_existing_user_is_rejected() {
    let app = spawn_app().await;

    let username = Uuid::new_v4();
    let password = Uuid::new_v4();

    let response = reqwest::Client::new()
        .post(format!("{}/newsletters", &app.address))
        .basic_auth(username, Some(password))
        .json(&serde_json::json!({
            "title": "Letter title",
            "content": {
                "html": "<a>Some link</a>",
                "text": "Some text"
            }
        }))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(401, response.status().as_u16());
    assert_eq!(
        r#"Basic realm="publish""#,
        response.headers()["WWW-Authenticate"]
    );
}

#[tokio::test]
async fn invalid_password_is_rejected() {
    let app = spawn_app().await;

    let username = &app.test_user.username;
    let password = Uuid::new_v4().to_string();

    assert_ne!(app.test_user.password, password);

    let response = reqwest::Client::new()
        .post(&format!("{}/newsletters", &app.address))
        .basic_auth(username, Some(password))
        .json(&serde_json::json!({
            "title": "Letter title",
            "content": {
                "html": "<a>Some link</a>",
                "text": "Some text"
            }
        }))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(401, response.status().as_u16());
    assert_eq!(
        r#"Basic realm="publish""#,
        response.headers()["WWW-Authenticate"]
    );
}
