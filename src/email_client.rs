use config::builder;
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
struct SendEmailPayload {
    from: String,
    to: String,
    subject: String,
    html_body: String,
    text_body: String,
}

impl EmailClient {
    pub fn new(base_url: String, sender: SubscriberEmail, auth_token: Secret<String>) -> Self {
        Self {
            http_client: Client::new(),
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
            from: self.sender.as_ref().to_owned(),
            to: self.sender.as_ref().to_owned(),
            subject: subject.to_owned(),
            html_body: html_content.to_owned(),
            text_body: text_content.to_owned(),
        };

        self.http_client
            .post(email_api)
            .header(
                "X-Some-Server-Token",
                self.email_service_auth_token.expose_secret(),
            )
            .json(&payload)
            .send()
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use fake::{
        faker::{
            internet::en::SafeEmail,
            lorem::en::{Paragraph, Sentence, Word},
        },
        Fake,
    };
    use secrecy::Secret;
    use wiremock::{
        matchers::{header, header_exists, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use crate::domain::subscriber_email::SubscriberEmail;
    use crate::email_client::EmailClient;

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
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Setup email received and email content
        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..2).fake();

        // Act
        let _ = email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;

        // Assert
    }
}
