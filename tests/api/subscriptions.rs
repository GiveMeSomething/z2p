use reqwest::Client;

use crate::helpers::spawn_server;

#[tokio::test]
async fn subscribe_200_for_valid_form() {
    let config = spawn_server().await;
    let server_address = config.0;
    let db_config = config.1;
    let db_pool = db_config.database.pg_connection_pool();

    let test_client = Client::new();

    let test_name = "test";
    let test_email = "test@gmail.com";

    let body = format!("name={}&email={}", test_name, test_email);
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
        .fetch_one(&db_pool)
        .await
        .expect("Failed to query from the datadabase");

    assert_eq!(saved_subscription.name, test_name);
    assert_eq!(saved_subscription.email, test_email);
}

#[tokio::test]
async fn subscribe_400_for_invalid_form() {
    let config = spawn_server().await;
    let server_address = config.0;

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

#[tokio::test]
async fn subscribe_400_for_invalid_payload() {
    let config = spawn_server().await;
    let server_address = config.0;

    let test_client = Client::new();

    let test_cases = vec![
        ("name=&email=example@example.com", "empty name"),
        ("name=      &email=example@example.com", "whitespace name"),
        ("name=&email=example@example.com", "name with special characters"),
        ("name=thisisaverylongexample190237910273901230912730912730127301297301273091273097120397109371027312093710273012730129730127302173012731203917301723071203thisisaverylongexample190237910273901230912730912730127301297301273091273097120397109371027312093endofverylongexample&email=example@example.com", "too long name"),   
        ("name=hello&email=", "empty email"),
        ("name=hello&email=definately-not-a-valid-email", "invalid email")
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
