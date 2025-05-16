use crate::domain::SubscriberEmail;

pub struct EmailClient {
    sender: SubscriberEmail,
}

impl EmailClient {
    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: String,
        html_content: String,
        text_content: String,
    ) -> Result<(), String> {
        todo!()
    }
}