use std::net::TcpListener;

use once_cell::sync::Lazy;
use z2p::{
    configurations::{read_configuration, Settings},
    email_client::EmailClient,
    startup::run,
    telemetry::{gen_subscriber, init_subscriber},
};

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    // Enable logging to stdout if TEST_LOG=true
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = gen_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = gen_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

/**
* Function to spawn server (at the start of each tests)
*/
pub async fn spawn_server() -> (String, Settings) {
    Lazy::force(&TRACING);

    let mut configurations = read_configuration().expect("Failed to read configurations");

    // Setup random database for testing
    let db_pool = configurations.database.pg_connection_pool_random().await;

    // TODO: Patch this later for testing
    let email_sender = configurations
        .email_client
        .sender()
        .expect("Invalid sender's email address");
    let email_client = EmailClient::new(
        configurations.email_client.base_url.clone(),
        email_sender,
        configurations.email_client.auth_token.clone(),
        // Keep timeout low to avoid hanging tests
        std::time::Duration::from_secs(1),
    );

    let listener = TcpListener::bind("localhost:0")
        .unwrap_or_else(|err| panic!("Cannot bind to random port with error {:?}", err));
    let bind_port = listener.local_addr().unwrap().port();

    let server = run(listener, db_pool, email_client)
        .await
        .expect("Failed to spawn new server");

    tokio::spawn(server);

    (format!("http://localhost:{}", bind_port), configurations)
}
