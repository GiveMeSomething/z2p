use std::net::TcpListener;
use z2p::{
    configurations,
    email_client::EmailClient,
    startup::run,
    telemetry::{gen_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = gen_subscriber("z2p".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configurations =
        configurations::read_configuration().expect("Failed to read configurations.");

    // Setup database connection pool
    let db_connection_pool = configurations.database.pg_connection_pool();

    // Setup email client (also a connection pool underneath)
    let email_sender = configurations
        .email_client
        .sender()
        .expect("Invalid sender's email address");
    let email_client = EmailClient::new(
        configurations.email_client.base_url,
        email_sender,
        configurations.email_client.auth_token,
        std::time::Duration::from_micros(configurations.email_client.timeout),
    );

    let app_address = format!(
        "{}:{}",
        configurations.application.host, configurations.application.port
    );
    println!("Starting server at http://{}", app_address);

    let listener =
        TcpListener::bind(app_address).expect("Failed to create TCP listener on port 8000");
    run(listener, db_connection_pool, email_client)
        .await
        .unwrap()
        .await
}
