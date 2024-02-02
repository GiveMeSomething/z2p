use actix_http::Request;
use actix_web::{
    dev::{Service, ServiceResponse},
    test, web, App, Error, HttpResponse, Responder,
};

async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

pub async fn spawn_app() -> impl Service<Request, Response = ServiceResponse, Error = Error> {
    test::init_service(App::new().route("health_check", web::get().to(health_check))).await
}
