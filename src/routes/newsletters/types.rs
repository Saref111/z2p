use crate::domain::SubscriberEmail;

#[derive(serde::Deserialize)]
pub struct BodySchema {
    pub title: String,
    pub content: Content,
}

#[derive(serde::Deserialize)]
pub struct Content {
    pub text: String,
    pub html: String,
}

pub struct ConfirmedSubscriber {
    pub email: SubscriberEmail,
}
