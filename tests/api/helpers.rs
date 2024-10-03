use once_cell::sync::Lazy;
use uuid::Uuid;
use z2p::{
    configurations::{read_configuration, Settings},
    startup::Application,
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

    let configurations = {
        let mut config = read_configuration().expect("Failed to read configurations");

        // Mock to random db
        config.database.database_name = Uuid::new_v4().to_string();

        // Mock random OS port
        config.application.port = 0;

        config
    };

    configurations.database.configure_database().await;

    let application = Application::build(&configurations)
        .await
        .expect("Failed to build application");

    let address = format!("http://127.0.0.1:{}", application.port());

    tokio::spawn(application.run_until_stopped());

    (address, configurations)
}
