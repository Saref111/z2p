use wiremock::{
    Mock, ResponseTemplate,
    matchers::{any, path},
};

use super::helpers::{create_unconfirmed_subscriber, spawn_app};

#[tokio::test]
async fn unconfirmed_subscriber_doesnt_get_a_newsletter() {
    let app = spawn_app().await;
    create_unconfirmed_subscriber(&app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.email_server)
        .await;

    let body = serde_json::json!(
    {
        "from": {
            "email": "z2pnoreply@test.com"
        },
        "to": [{
            "email": "someemail@test.com"
        }],
        "html": "HTML content",
        "text": "text content",
        "subject": "subject"
    });

    let response = reqwest::Client::new()
        .post(format!("{}/newsletters", &app.address))
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 200);
}
