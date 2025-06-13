use std::time::Duration;

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
    pub fn new(
        base_url: String,
        sender: SubscriberEmail,
        auth_token: SecretString,
        timeout: Duration,
    ) -> Self {
        Self {
            http_client: Client::builder().timeout(timeout).build().unwrap(),
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

        let mr = self.http_client
            .post(url)
            .header(
                "Authorization",
                "Bearer ".to_owned() + self.auth_token.expose_secret(),
            )
            .json(&body)
            .send()
            .await?;
            // .error_for_status()?;
        dbg!(mr);

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use claims::{assert_err, assert_ok};
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

    fn get_subject() -> String {
        Sentence(1..2).fake()
    }

    fn get_content() -> String {
        Paragraph(1..10).fake()
    }

    fn get_email() -> SubscriberEmail {
        SubscriberEmail::parse(SafeEmail().fake()).unwrap()
    }

    fn get_email_client(base_url: String) -> EmailClient {
        EmailClient::new(
            base_url,
            get_email(),
            SecretString::from(Faker.fake::<String>()),
            Duration::from_millis(10),
        )
    }

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        let mock_server = MockServer::start().await;
        let email_client = get_email_client(mock_server.uri());

        Mock::given(header_exists("Authorization"))
            .and(header("Content-type", "application/json"))
            .and(path("v1/email"))
            .and(method("POST"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let subscriber_email = get_email();
        let subject: String = get_subject();
        let content: String = get_content();

        let _ = email_client
            .send_email(subscriber_email, subject, &content, &content)
            .await;
    }

    #[tokio::test]
    async fn send_email_succeeds_if_server_returns_200() {
        let mock_server = MockServer::start().await;
        let email_client = get_email_client(mock_server.uri());

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let subscriber_email = get_email();
        let subject: String = get_subject();
        let content: String = get_content();

        let outcome = email_client
            .send_email(subscriber_email, subject, &content, &content)
            .await;

        assert_ok!(outcome)
    }

    #[tokio::test]
    async fn send_email_fails_if_server_returns_500() {
        let mock_server = MockServer::start().await;
        let email_client = get_email_client(mock_server.uri());

        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        let subscriber_email = get_email();
        let subject: String = get_subject();
        let content: String = get_content();

        let outcome = email_client
            .send_email(subscriber_email, subject, &content, &content)
            .await;

        assert_err!(outcome);
        ()
    }

    #[tokio::test]
    async fn send_email_times_out_if_server_takes_too_long() {
        let mock_server = MockServer::start().await;
        let email_client = get_email_client(mock_server.uri());

        let response = ResponseTemplate::new(500).set_delay(Duration::from_secs(20));
        Mock::given(any())
            .respond_with(response)
            .expect(1)
            .mount(&mock_server)
            .await;

        let subscriber_email = get_email();
        let subject: String = get_subject();
        let content: String = get_content();

        let outcome = email_client
            .send_email(subscriber_email, subject, &content, &content)
            .await;

        assert_err!(outcome);
        ()
    }
}
