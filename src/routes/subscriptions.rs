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
        Ok(_) => HttpResponse::Ok(),
        Err(error) => {
            println!("Failed to execute query with error {}", error);
            HttpResponse::InternalServerError()
        }
    }
}
