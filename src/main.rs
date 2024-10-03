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
}
