use std::net::TcpListener;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};
use z2p::{configurations, startup::run};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Redirect all log to subscriber
    LogTracer::init().expect("Failed to set logger");

    // Setup logger
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let formatting_layer = BunyanFormattingLayer::new("z2p".into(), std::io::stdout);

    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);

    set_global_default(subscriber).expect("Failed to set subscriber");

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
