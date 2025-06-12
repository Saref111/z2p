use wiremock::{
    Mock, ResponseTemplate,
    matchers::{method, path},
};

use crate::helpers::spawn_app;

#[tokio::test]
async fn confirmation_without_token_rejected_with_400() {
    let app = spawn_app().await;

    let resp = reqwest::get(&format!("{}/subscriptions/confirm", &app.address))
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 400);
}

#[tokio::test]
async fn link_returned_by_subscribe_returns_200_if_called() {
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("v1/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscription(body.into()).await;
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    

    let confirmation_links = app.get_confirmation_links(email_request);
    let resp = reqwest::get(confirmation_links.html).await.unwrap();

    assert_eq!(resp.status().as_u16(), 200);
}
