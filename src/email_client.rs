use std::str::FromStr;

use reqwest::{Client, Url};
use secrecy::{ExposeSecret, SecretString};
use serde::Serialize;

use crate::domain::SubscriberEmail;

#[derive(Clone)]
pub struct EmailClient {
    http_client: Client,
    base_url: Url,
    sender: SubscriberEmail,
    auth_token: SecretString,
}

#[derive(Serialize)]
struct SendEmailRequest<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    html: &'a str,
    text: &'a str,
}

impl EmailClient {
    pub fn new(base_url: String, sender: SubscriberEmail, auth_token: SecretString) -> Self {
        Self {
            http_client: Client::new(),
            base_url: Url::parse(&base_url).expect("Failed parsing base email api url."),
            sender,
            auth_token,
        }
    }

    /*

    curl ...  --data '{
      "to": [
        {
          "email": "saref012@gmail.com"
        }
      ],
      "from": {
        "email": "noreply@test-r83ql3p28rmgzw1j.mlsender.net"
      },
      "subject": "New Attachment Test",
      "html": "<p>Hello world!</p>"
    }'


     */

    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: String,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), reqwest::Error> {
        let url = self
            .base_url
            .join("v1/email")
            .expect("Failed joining route to email api url.");

        let body = SendEmailRequest {
            from: self.sender.as_ref(),
            to: recipient.as_ref(),
            html: html_content,
            text: text_content,
            subject: &subject,
        };

        self.http_client
            .post(url)
            .header(
                "Authorization",
                "Bearer: ".to_owned() + self.auth_token.expose_secret(),
            )
            .json(&body)
            .send()
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use fake::{
        Fake, Faker,
        faker::{
            internet::en::SafeEmail,
            lorem::en::{Paragraph, Sentence},
        },
    };
    use secrecy::SecretString;
    use wiremock::{Mock, MockServer, ResponseTemplate, matchers::any};

    use crate::{domain::SubscriberEmail, email_client::EmailClient};

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        let mock_server = MockServer::start().await;
        let sender = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let auth_token: String = Faker.fake();
        let email_client =
            EmailClient::new(mock_server.uri(), sender, SecretString::from(auth_token));

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        let _ = email_client
            .send_email(subscriber_email, subject, &content, &content)
            .await;
    }
}
