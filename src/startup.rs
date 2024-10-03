use std::net::TcpListener;

use actix_http::Request;
use actix_web::{
    dev::{Server, Service, ServiceResponse},
    test, web, App, Error, HttpServer,
};
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

use crate::{
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
