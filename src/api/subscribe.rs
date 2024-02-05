use actix_web::{web::Form, HttpResponse, Responder};

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

pub async fn subscribe(_form: Form<FormData>) -> impl Responder {
    println!("Name: {}, Email: {}", _form.name, _form.email);
    HttpResponse::Ok()
}
