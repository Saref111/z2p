use uuid::Uuid;

use crate::helpers::{assert_is_redirect_to, spawn_app};

#[tokio::test]
async fn you_must_be_logged_in_to_see_change_password_form() {
    let app = spawn_app().await;

    let resp = app.get_change_password().await;

    assert_is_redirect_to(&resp, "/login");
}

#[tokio::test]
async fn you_must_be_logged_in_to_change_your_password() {
    let app = spawn_app().await;
    let new_password = Uuid::new_v4().to_string();

    let resp = app
        .post_change_password(&serde_json::json!({
            "current_password": &app.test_user.password,
            "new_password": &new_password,
            "new_password_check": &new_password
        }))
        .await;

    assert_is_redirect_to(&resp, "/login");
}

#[tokio::test]
async fn new_password_fields_must_match() {
    let app = spawn_app().await;
    let new_password = Uuid::new_v4().to_string();
    let new_password_check = Uuid::new_v4().to_string();

    app.post_login(&serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password
    }))
    .await;

    let resp = app
        .post_change_password(&serde_json::json!({
            "current_password": &app.test_user.password,
            "new_password": &new_password,
            "new_password_check": &new_password_check,
        }))
        .await;

    assert_is_redirect_to(&resp, "/admin/password");

    let html_page = app.get_change_password_html().await;
    assert!(html_page.contains(
        "You entered two different new passwords - \
        the field values must match."
    ));
}

#[tokio::test]
async fn current_password_must_be_valid() {
    let app = spawn_app().await;
    let new_password = Uuid::new_v4().to_string();
    let wrong_password = Uuid::new_v4().to_string();

    app.post_login(&serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password
    }))
    .await;

    let resp = app
        .post_change_password(&serde_json::json!({
            "current_password": &wrong_password,
            "new_password": &new_password,
            "new_password_check": &new_password
        }))
        .await;

    assert_is_redirect_to(&resp, "/admin/password");

    let html_page = app.get_change_password_html().await;
    assert!(html_page.contains("The current password is incorrect."));
}

#[tokio::test]
async fn changing_password_works() {
    let app = spawn_app().await;
    let new_password = Uuid::new_v4().to_string();

    let resp = app
        .post_login(&serde_json::json!({
            "username": &app.test_user.username,
            "password": &app.test_user.password
        }))
        .await;
    assert_is_redirect_to(&resp, "/admin/dashboard");

    let resp = app
        .post_change_password(&serde_json::json!({
            "current_password": &app.test_user.password,
            "new_password": &new_password,
            "new_password_check": &new_password
        }))
        .await;
    assert_is_redirect_to(&resp, "/admin/password");

    let html_page = app.get_change_password_html().await;
    assert!(html_page.contains("Your password has been changed."));

    let resp = app.post_logout().await;
    assert_is_redirect_to(&resp, "/login");

    let html_page = app.get_login_html().await;
    assert!(html_page.contains("You have successfully logged out."));

    let resp = app
        .post_login(&serde_json::json!({
            "username": &app.test_user.username,
            "password": &new_password
        }))
        .await;
    assert_is_redirect_to(&resp, "/admin/dashboard");
}
