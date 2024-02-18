use std::net::TcpListener;
use z2p::{configurations, startup::run};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configurations =
        configurations::read_configuration().expect("Failed to read configurations.");

    println!(
        "Starting server with database {}",
        configurations.database.connection_string()
    );

    let db_connection_pool = configurations.database.pg_connection_pool().await;

    let app_address = format!("localhost:{}", configurations.app_port);
    println!("Starting server at http://{}", app_address);

    let listener =
        TcpListener::bind(app_address).expect("Failed to create TCP listener on port 8000");

    run(listener, db_connection_pool).await.unwrap().await
}
