use std::net::TcpListener;
use z2p::run;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let listener =
        TcpListener::bind("localhost:8000").expect("Failed to create TCP listener on port 8000");
    run(listener).await.unwrap().await
}
