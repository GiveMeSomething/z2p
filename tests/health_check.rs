use actix_http::body::MessageBody;
use actix_web::{body::BodySize, test};
use reqwest::Client;
use z2p::startup::{spawn_app, spawn_server};

#[tokio::test]
async fn health_check_works() {
    let app = spawn_app().await;

    let req = test::TestRequest::get().uri("/health_check").to_request();
    let res = test::call_service(&app, req).await;

    assert!(res.status().is_success());
    assert_eq!(res.into_body().size(), BodySize::Sized(0));
}

#[tokio::test]
async fn health_check_works_reqwest() {
    // Ignore warning, tokio manage the server in a different thread
    let server_address = spawn_server().await;
    let test_client = Client::new();

    let response = test_client
        .get(format!("{}/health_check", server_address))
        .send()
        .await
        .expect("Failed to send the request to server");

    assert!(response.status().is_success());
    assert_eq!(response.content_length(), Some(0));
}
