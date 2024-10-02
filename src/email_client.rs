use reqwest::{Client, Url};
use secrecy::{ExposeSecret, Secret};
use serde::Serialize;

use crate::domain::subscriber_email::SubscriberEmail;

pub struct EmailClient {
    http_client: Client,
    base_url: String,
    sender: SubscriberEmail,

    // We don't want to get this into log by accident
    email_service_auth_token: Secret<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailPayload<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    html_body: &'a str,
    text_body: &'a str,
}

impl EmailClient {
    pub fn new(base_url: String, sender: SubscriberEmail, auth_token: Secret<String>) -> Self {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap();
        Self {
            http_client,
            base_url,
            sender,
            email_service_auth_token: auth_token,
        }
    }

    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), reqwest::Error> {
        let base_url = Url::parse(&self.base_url).expect("Invalid email client's base url");
        let email_api = base_url.join("/email").expect("Invalid email request API");

        let payload = SendEmailPayload {
            from: self.sender.as_ref(),
            to: recipient.as_ref(),
            subject,
            html_body: html_content,
            text_body: text_content,
        };

        self.http_client
            .post(email_api)
            .header(
                "X-Some-Server-Token",
                self.email_service_auth_token.expose_secret(),
            )
            .json(&payload)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use claims::{assert_err, assert_ok};
    use fake::{
        faker::{
            internet::en::SafeEmail,
            lorem::en::{Paragraph, Sentence, Word},
        },
        Fake,
    };
    use secrecy::Secret;
    use wiremock::{
        matchers::{any, header, header_exists, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use crate::domain::subscriber_email::SubscriberEmail;
    use crate::email_client::EmailClient;

    struct SendEmailPayloadMatcher;

    impl wiremock::Match for SendEmailPayloadMatcher {
        fn matches(&self, request: &wiremock::Request) -> bool {
            // Try to parse the payload as JSON
            match serde_json::from_slice::<serde_json::Value>(&request.body) {
                Ok(body) => {
                    body.get("From").is_some()
                        && body.get("To").is_some()
                        && body.get("HtmlBody").is_some()
                        && body.get("TextBody").is_some()
                }
                Err(_) => false,
            }
        }
    }

    #[tokio::test]
    async fn send_email_send_request() {
        // Create a new HTTP server with wiremock
        let mock_server = MockServer::start().await;

        // Mock email client with new email sender
        let sender = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let email_client = EmailClient::new(mock_server.uri(), sender, Secret::new(Word().fake()));

        // Setup mock server
        Mock::given(header_exists("X-Some-Server-Token"))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            .and(SendEmailPayloadMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Setup email received and email content
        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..2).fake();

        // Act
        let result = email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;

        // Assert
        assert_ok!(result);
    }

    #[tokio::test]
    async fn send_email_fails_if_server_response_not_ok() {
        // Create a new HTTP server with wiremock
        let mock_server = MockServer::start().await;

        // Mock email client with new email sender
        let sender = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let email_client = EmailClient::new(mock_server.uri(), sender, Secret::new(Word().fake()));

        // Setup mock server
        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Setup email received and email content
        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..2).fake();

        // Act
        let result = email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;

        // Assert
        assert_err!(result);
    }

    async fn send_email_fails_if_server_hang() {
        // Create a new HTTP server with wiremock
        let mock_server = MockServer::start().await;

        // Mock email client with new email sender
        let sender = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let email_client = EmailClient::new(mock_server.uri(), sender, Secret::new(Word().fake()));

        // Setup mock server
        Mock::given(any())
            .respond_with(ResponseTemplate::new(500).set_delay(std::time::Duration::from_secs(60)))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Setup email received and email content
        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..2).fake();

        // Act
        let result = email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;

        // Assert
        assert_err!(result);
    }
}
