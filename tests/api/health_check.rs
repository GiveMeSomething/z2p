use reqwest::Client;

use crate::helpers::spawn_server;

#[tokio::test]
async fn health_check_works_reqwest() {
    // Ignore warning, tokio manage the server in a different thread
    let config = spawn_server().await;
    let server_address = config.0;

    let test_client = Client::new();

    let response = test_client
        .get(format!("{}/health_check", server_address))
        .send()
        .await
        .expect("Failed to send the request to server");

    assert!(response.status().is_success());
    assert_eq!(response.content_length(), Some(0));
}
