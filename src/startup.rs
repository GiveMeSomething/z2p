use std::net::TcpListener;

use actix_http::Request;
use actix_web::{
    dev::{Server, Service, ServiceResponse},
    test, web, App, Error, HttpServer,
};
use once_cell::sync::Lazy;
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

use crate::{
    configurations::{read_configuration, Settings},
    email_client::EmailClient,
    routes::{health_check, subscribe},
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
 * Actix provide some conveniences to interact with an App without skipping the routing logic
 *
 * But,
 *
 * 1. Migrating to another web framework would force us to rewrite our whole integration test suite.
 *    And we'd like our integration tests ot be highly decoupled
 *
 * 2. Due to some limitations, we cannot share the App startup logic between our production and test logic
 * => Risk of divergence over time
 *
 * Therefore, we will opt for a fully black-box solution,
 * we will launch our application at the beginning of each tests and interact with it using a HTTP Client
 */
pub async fn spawn_app() -> impl Service<Request, Response = ServiceResponse, Error = Error> {
    test::init_service(App::new().route("health_check", web::get().to(health_check))).await
}

/**
* Function to spawn server (at the start of each tests)
*/
pub async fn spawn_server() -> (String, Settings) {
    Lazy::force(&TRACING);

    let mut configurations = read_configuration().expect("Failed to read configurations");

    // Setup random database for testing
    let db_pool = configurations.database.pg_connection_pool_random().await;

    // TODO: Patch this later for testing
    let email_sender = configurations
        .email_client
        .sender()
        .expect("Invalid sender's email address");
    let email_client = EmailClient::new(
        configurations.email_client.base_url.clone(),
        email_sender,
        configurations.email_client.auth_token.clone(),
    );

    let listener = TcpListener::bind("localhost:0")
        .unwrap_or_else(|err| panic!("Cannot bind to random port with error {:?}", err));
    let bind_port = listener.local_addr().unwrap().port();

    let server = run(listener, db_pool, email_client)
        .await
        .expect("Failed to spawn new server");

    tokio::spawn(server);

    (format!("http://localhost:{}", bind_port), configurations)
}

pub async fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
) -> Result<Server, std::io::Error> {
    // Atomic Reference Counted pointer - smart pointer
    let db_pool = web::Data::new(db_pool);
    let email_client = web::Data::new(email_client);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("health_check", web::get().to(health_check))
            .route("subscriptions", web::post().to(subscribe))
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
