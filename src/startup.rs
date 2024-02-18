use std::net::TcpListener;

use actix_http::Request;
use actix_web::{
    dev::{Server, Service, ServiceResponse},
    test, web, App, Error, HttpServer,
};
use sqlx::PgPool;

use crate::{
    configurations::read_configuration,
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

/**
* Function to spawn server (at the start of each tests)
*/
pub async fn spawn_server() -> String {
    let configurations = read_configuration().expect("Failed to read configurations");
    let db_pool = configurations.database.pg_connection_pool().await;

    let listener = TcpListener::bind("localhost:0")
        .unwrap_or_else(|err| panic!("Cannot bind to random port with error {:?}", err));
    let bind_port = listener.local_addr().unwrap().port();

    let server = run(listener, db_pool)
        .await
        .expect("Failed to spawn new server");
    let _ = tokio::spawn(server);

    format!("http://localhost:{}", bind_port)
}

pub async fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    // Atomic Reference Counted pointer - smart pointer
    let db_pool = web::Data::new(db_pool);

    let server = HttpServer::new(move || {
        App::new()
            .route("health_check", web::get().to(health_check))
            .route("subscriptions", web::post().to(subscribe))
            .app_data(db_pool.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
