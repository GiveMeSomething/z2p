use reqwest::Client;
use z2p::startup::spawn_server;

#[tokio::test]
async fn subscribe_200_for_valid_form() {
    let server_address = spawn_server().await;
    let test_client = Client::new();

    let body = "name=test&email=test@gmail.com";
    let response = test_client
        .post(format!("{}/subscriptions", server_address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to send the request to the server");

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn subscribe_400_for_invalid_form() {
    let server_address = spawn_server().await;
    let test_client = Client::new();

    let test_cases = vec![
        ("name=test", "missing email case"),
        ("email=test@testing.com", "missing name case"),
        ("", "missing name and email case"),
    ];

    for (invalid_body, message) in test_cases {
        let response = test_client
            .post(format!("{}/subscriptions", server_address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to send the request to the server");

        assert_eq!(
            response.status().as_u16(),
            400,
            "The API did not fail with 400 status code, case: {}",
            message
        );
    }
}
