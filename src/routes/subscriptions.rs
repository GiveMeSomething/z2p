use std::ops::DerefMut;

use actix_web::{
    web::{self, Form},
    HttpResponse, Responder,
};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use sqlx::{types::chrono::Utc, PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::domain::new_subscriber::NewSubscriber;

#[derive(serde::Deserialize)]
pub struct FormData {
    pub name: String,
    pub email: String,
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, db_pool),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(form: Form<FormData>, db_pool: web::Data<PgPool>) -> impl Responder {
    // If you provide a TryFrom implementation, your type automatically gets the corresponding TryInto implementation, for free
    let new_subscriber: NewSubscriber = match form.0.try_into() {
        Ok(subscriber) => subscriber,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    let mut tx = match db_pool.begin().await {
        Ok(tx) => tx,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    // Create new subscriber
    let subscriber_id = match insert_subscriber(&mut tx, &new_subscriber).await {
        Ok(id) => id,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let subscription_token = generate_subscription_token();
    match insert_subscription_token(&mut tx, subscriber_id, &subscription_token).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(tx, new_subscriber)
)]
async fn insert_subscriber(
    tx: &mut Transaction<'_, Postgres>,
    new_subscriber: &NewSubscriber,
) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO subscriptions (id, name, email, subscribed_at) VALUES ($1, $2, $3, $4)",
        subscriber_id,
        new_subscriber.name.as_ref(),
        new_subscriber.email.as_ref(),
        Utc::now(),
    )
    // This extract the inner connection from the tx, which is required for this execute function to work
    .execute(tx.deref_mut())
    .await
    .map_err(|err| {
        tracing::error!("Failed to execute query: {:?}", err);
        err
    })?;

    Ok(subscriber_id)
}

async fn insert_subscription_token(
    tx: &mut Transaction<'_, Postgres>,
    subscriber_id: Uuid,
    subscription_token: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO subscription_tokens (subscription_token, subscriber_id) VALUES ($1, $2)"#,
        subscription_token,
        subscriber_id
    )
    .execute(tx.deref_mut())
    .await
    .map_err(|err| {
        tracing::error!("Failed to execute query: {:?}", err);
        err
    })?;

    Ok(())
}

fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}
