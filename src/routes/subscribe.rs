use actix_web::{web::Form, HttpResponse, Responder};

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

pub async fn subscribe(form: Form<FormData>) -> impl Responder {
    HttpResponse::Ok().body(format!("Name: {}, Email: {}", form.name, form.email))
}
