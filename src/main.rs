use std::net::TcpListener;
use z2p::{configurations, startup::run};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configurations =
        configurations::read_configuration().expect("Failed to read configurations.");

    let db_config = configurations.database;
    println!(
        "Starting server with database {}",
        format!(
            "postgres://{}:{}@{}:{}/{}",
            db_config.username,
            db_config.password,
            db_config.host,
            db_config.port,
            db_config.database_name
        )
    );

    let app_address = format!("localhost:{}", configurations.app_port);
    println!("Starting server at http://{}", app_address);

    let listener =
        TcpListener::bind(app_address).expect("Failed to create TCP listener on port 8000");

    run(listener).await.unwrap().await
}
