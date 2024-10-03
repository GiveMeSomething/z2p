use std::net::TcpListener;

use actix_http::Request;
use actix_web::{
    dev::{Server, Service, ServiceResponse},
    test, web, App, Error, HttpServer,
};
use secrecy::ExposeSecret;
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

use crate::{
    configurations::Settings,
    email_client::EmailClient,
    routes::{health_check, subscribe},
};

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

pub fn build_email_client(config: &Settings) -> EmailClient {
    let sender = config
        .email_client
        .sender()
        .expect("Invalid sender's email");
    let timeout = config.email_client.timeout;
    EmailClient::new(
        config.email_client.base_url.to_owned(),
        sender,
        config.email_client.auth_token.to_owned(),
        std::time::Duration::from_millis(timeout),
    )
}

pub fn build_connection_pool(config: &Settings) -> PgPool {
    PgPool::connect_lazy(config.database.connection_string().expose_secret()).unwrap_or_else(
        |err| {
            panic!(
                "Failed to connect to the datbase connection pool with error {:?}",
                err
            )
        },
    )
}

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(config: &Settings) -> Result<Self, std::io::Error> {
        let db_pool = build_connection_pool(&config);
        let email_client = build_email_client(&config);

        let address = format!(
            "{}:{}",
            config.application.host.to_owned(),
            config.application.port.to_owned()
        );
        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr().unwrap().port();
        let server = run(listener, db_pool, email_client).await?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    // Consume self
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
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
