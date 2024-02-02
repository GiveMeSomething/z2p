use std::net::TcpListener;

use actix_http::Request;
use actix_web::{
    dev::{Server, Service, ServiceResponse},
    test, web, App, Error, HttpResponse, HttpServer, Responder,
};

async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

pub async fn spawn_app() -> impl Service<Request, Response = ServiceResponse, Error = Error> {
    test::init_service(App::new().route("health_check", web::get().to(health_check))).await
}

pub async fn spawn_server() -> String {
    let listener = TcpListener::bind("localhost:0")
        .unwrap_or_else(|err| panic!("Cannot bind to random port with error {:?}", err));
    let bind_port = listener.local_addr().unwrap().port();

    let server = run(listener).await.expect("Failed to spawn new server");
    let _ = tokio::spawn(server);

    format!("http://localhost:{}", bind_port)
}

pub async fn run(listener: TcpListener) -> std::io::Result<Server> {
    let server = HttpServer::new(|| App::new().route("health_check", web::get().to(health_check)))
        .listen(listener)?
        .run();

    Ok(server)
}
