use actix_web::{
    web::{self, Form},
    HttpResponse, Responder,
};
use sqlx::{types::chrono::Utc, PgPool};
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

pub async fn subscribe(form: Form<FormData>, db_pool: web::Data<PgPool>) -> impl Responder {
    let request_id = Uuid::new_v4();

    let request_span = tracing::info_span!(
        "Adding a new subscriber.",
        %request_id,
        sub_email = %form.email,
        sub_name = %form.name
    );

    let _request_span_guard = request_span.enter();

    tracing::info!(
        "request_id {} - Adding user with email: {}, name: {}",
        request_id,
        form.email,
        form.name
    );

    match sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, name, email, subscribed_at)
    VALUES ($1, $2, $3, $4)
    "#,
        Uuid::new_v4(),
        form.name,
        form.email,
        Utc::now()
    )
    .execute(db_pool.get_ref())
    .await
    {
        Ok(_) => {
            tracing::info!(
                "request_id {} - New subscriber details have been saved",
                request_id
            );
            HttpResponse::Ok()
        }
        Err(error) => {
            tracing::error!(
                "request_id {} - Failed to execute query with error {:?}",
                request_id,
                error
            );
            HttpResponse::InternalServerError()
        }
    }
}