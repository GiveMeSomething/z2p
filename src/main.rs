use std::net::TcpListener;
use z2p::{configurations, startup::run};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configurations =
        configurations::read_configuration().expect("Failed to read configurations.");

    let app_address = format!("localhost:{}", configurations.app_port);
    let listener =
        TcpListener::bind(app_address).expect("Failed to create TCP listener on port 8000");

    run(listener).await.unwrap().await
}
