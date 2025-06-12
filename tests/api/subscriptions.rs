use wiremock::{
    Mock, ResponseTemplate,
    matchers::{method, path},
};

use crate::helpers::spawn_app;

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("v1/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let response = app.post_subscription(body.into()).await;

    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_persists_the_new_subscriber() {
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    app.post_subscription(body.into()).await;

    let saved = sqlx::query!("SELECT name, email, status FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com".to_string());
    assert_eq!(saved.name, "le guin".to_string());
    assert_eq!(saved.status, "pending_confirmation".to_string());
}

#[tokio::test]
async fn subscribe_returns_400_when_data_is_missing() {
    let app = spawn_app().await;

    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (data, err_message) in test_cases {
        let response = app.post_subscription(data.into()).await;

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            err_message
        );
    }
}

#[tokio::test]
async fn subscribe_returns_a_400_when_fields_are_present_but_empty() {
    let app = spawn_app().await;
    let test_cases = vec![
        ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
        ("name=Ursula&email=", "empty email"),
        ("name=Ursula&email=definitely-not-an-email", "invalid email"),
    ];

    for (body, description) in test_cases {
        let response = app.post_subscription(body.into()).await;

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a 400 Bad Request when the payload was {}.",
            description
        )
    }
}

#[tokio::test]
async fn subscribe_send_confirmation_email_for_valid_data() {
    let app = spawn_app().await;

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("v1/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    app.post_subscription(body.into()).await;
}

#[tokio::test]
async fn subscribe_send_confirmation_email_with_link() {
    let app = spawn_app().await;

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("v1/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscription(body.into()).await;

    let received_request = &app.email_server.received_requests().await.unwrap()[0];

    let confirmation_links = app.get_confirmation_links(received_request);

    assert_eq!(confirmation_links.html, confirmation_links.text);
}
