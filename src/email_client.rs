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
struct EmailUnit<'a> {
    email: &'a str,
}

impl<'a> EmailUnit<'a> {
    fn new(email: &'a str) -> Self {
        Self { email }
    }
}

#[derive(Serialize)]
struct SendEmailRequest<'a> {
    from: EmailUnit<'a>,
    to: Vec<EmailUnit<'a>>,
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
            from: EmailUnit::new(self.sender.as_ref()),
            to: vec![EmailUnit::new(recipient.as_ref())],
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
            .await?
            .error_for_status()?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use claim::{assert_err, assert_ok};
    use fake::{
        Fake, Faker,
        faker::{
            internet::en::SafeEmail,
            lorem::en::{Paragraph, Sentence},
        },
    };
    use secrecy::SecretString;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{any, header, header_exists, method, path},
    };

    use crate::{domain::SubscriberEmail, email_client::EmailClient};

    struct SendEmailBodyMatcher;

    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &wiremock::Request) -> bool {
            let result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);

            if let Ok(body) = result {
                body.get("from").is_some()
                    && body.get("to").is_some()
                    && body.get("subject").is_some()
                    && body.get("html").is_some()
                    && body.get("text").is_some()
            } else {
                false
            }
        }
    }

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        let mock_server = MockServer::start().await;
        let sender = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let auth_token: String = Faker.fake();
        let email_client =
            EmailClient::new(mock_server.uri(), sender, SecretString::from(auth_token));

        Mock::given(header_exists("Authorization"))
            .and(header("Content-type", "application/json"))
            .and(path("v1/email"))
            .and(method("POST"))
            .and(SendEmailBodyMatcher)
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

    #[tokio::test]
    async fn send_email_succeeds_if_server_returns_200() {
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

        let outcome = email_client
            .send_email(subscriber_email, subject, &content, &content)
            .await;

        assert_ok!(outcome)
    }

    #[tokio::test]
    async fn send_email_fails_if_server_returns_500() {
        let mock_server = MockServer::start().await;
        let sender = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let auth_token: String = Faker.fake();
        let email_client =
            EmailClient::new(mock_server.uri(), sender, SecretString::from(auth_token));

        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        let outcome = email_client
            .send_email(subscriber_email, subject, &content, &content)
            .await;

        assert_err!(outcome);
        ()
    }
}
