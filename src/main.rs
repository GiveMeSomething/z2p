use std::net::TcpListener;
use z2p::{
    configurations,
    startup::run,
    telemetry::{gen_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = gen_subscriber("z2p".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configurations =
        configurations::read_configuration().expect("Failed to read configurations.");

    let db_connection_pool = configurations.database.pg_connection_pool();

    let app_address = format!(
        "{}:{}",
        configurations.application.host, configurations.application.port
    );
    println!("Starting server at http://{}", app_address);

    let listener =
        TcpListener::bind(app_address).expect("Failed to create TCP listener on port 8000");
    run(listener, db_connection_pool).await.unwrap().await
}
