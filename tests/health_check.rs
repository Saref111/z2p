use std::net::TcpListener;

use reqwest;

#[tokio::test]
async fn health_check_works() {
    let url = spawn_app();

    let client = reqwest::Client::new();

    let response = client
        .get(url + "/health_check")
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port.");
    let port = listener.local_addr().unwrap().port();
    let server = z2p::run(listener).expect("Failed to bind address.");

    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{port}")
}

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let app_address = spawn_app();
    let client = reqwest::Client::new();

    let body = "username=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(app_address + "/subscriptions")
        .body(body)
        .header("Content-type", "application/x-www-form-urlencoded")
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_returns_400_when_data_is_missing() {
    let app_address = spawn_app();
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (data, err_message) in test_cases {
        let response = client
            .post(format!("{}/subscriptions", &app_address))
            .body(data)
            .header("Content-type", "application/x-www-form-urlencoded")
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            err_message
        );
    }
}
