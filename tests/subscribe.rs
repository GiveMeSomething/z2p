use reqwest::Client;
use z2p::{configurations::read_configuration, startup::spawn_server};

#[tokio::test]
async fn subscribe_200_for_valid_form() {
    let configuration = read_configuration().expect("Failed to read configuration");

    let mut connection = configuration.database.pg_connection().await;

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

    // Check server response
    assert_eq!(response.status().as_u16(), 200);

    // Check database data
    let saved_subscription = sqlx::query!("SELECT name, email FROM subscriptions")
        .fetch_one(&mut connection)
        .await
        .expect("Failed to query from the datadabase");

    assert_eq!(saved_subscription.name, "test");
    assert_eq!(saved_subscription.email, "test@gmail.com");
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
