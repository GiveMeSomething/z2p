use std::io::Result;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};

async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

pub async fn run() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/health_check", web::get().to(health_check)))
        .bind("localhost:8000")?
        .run()
        .await
}

#[actix_web::main]
async fn main() -> Result<()> {
    run().await
}
