use reqwest::Url;
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
    let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

    let get_link = |s: &str| {
        let links = linkify::LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == linkify::LinkKind::Url)
            .collect::<Vec<_>>();

        assert_eq!(links.len(), 1);
        links[0].as_str().to_owned()
    };

    let raw_confirmation_link = get_link(body["html"].as_str().unwrap());
    let mut confirmation_link = Url::parse(&raw_confirmation_link).unwrap();

    assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");

    confirmation_link.set_port(Some(app.port)).unwrap();
    let resp = reqwest::get(confirmation_link).await.unwrap();

    assert_eq!(resp.status().as_u16(), 200);
}
