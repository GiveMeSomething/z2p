use actix_web::{web, HttpResponse, Responder};

#[derive(serde::Deserialize)]
pub struct ConfirmPayload {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(_parameters))]
pub async fn confirm_subscription(_parameters: web::Query<ConfirmPayload>) -> impl Responder {
    println!(
        "Confirming user with subscription_token: {}",
        _parameters.into_inner().subscription_token
    );
    return HttpResponse::Ok().finish();
}
