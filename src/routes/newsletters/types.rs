use crate::domain::SubscriberEmail;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct BodySchema {
    pub title: String,
    pub text: String,
    pub html: String,
}

pub struct ConfirmedSubscriber {
    pub email: SubscriberEmail,
}
