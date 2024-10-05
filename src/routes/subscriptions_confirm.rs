use actix_web::{web, HttpResponse, Responder};

#[derive(serde::Deserialize)]
pub struct ConfirmPayload {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(_parameters))]
pub async fn confirm_subscription(_parameters: web::Query<ConfirmPayload>) -> impl Responder {
    return HttpResponse::Ok().finish();
}
