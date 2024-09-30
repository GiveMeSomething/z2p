use crate::domain::subscriber_email::SubscriberEmail;

pub struct EmailClient {
    sender: SubscriberEmail,
}

impl EmailClient {
    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_contet: &str,
    ) -> Result<(), String> {
        todo!()
    }
}
