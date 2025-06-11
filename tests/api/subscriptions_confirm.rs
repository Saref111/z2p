use crate::helpers::spawn_app;

#[tokio::test]
async fn confirmation_without_token_rejected_with_400() {
    let app = spawn_app().await;

    let resp = reqwest::get(&format!("{}/subscriptions/confirm", &app.address))
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 400);
}