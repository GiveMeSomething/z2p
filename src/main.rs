use z2p::{
    configurations,
    startup::Application,
    telemetry::{gen_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = gen_subscriber("z2p".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configurations =
        configurations::read_configuration().expect("Failed to read configurations.");

    let application = Application::build(&configurations)
        .await
        .expect("Failed to start server");

    application.run_until_stopped().await
}
