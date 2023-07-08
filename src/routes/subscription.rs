use crate::configuration::Db;
use crate::domain::subscriber_email::SubscriberEmail;
use crate::domain::subscriber_name::SubscriberName;
use crate::domain::NewSubscriber;
use axum::{http, response::IntoResponse, Form};
use sqlx::types::chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct FormData {
    email: String,
    name: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let email = SubscriberEmail::try_from(value.email.as_str())?;
        let name = SubscriberName::try_from(value.name.as_str())?;

        Ok(Self { name, email })
    }
}

#[allow(clippy::async_yields_async)]
#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, db),
    fields(
        subscriber_name = %form.name,
        subscriber_email = %form.email
    )
)]
pub async fn subscribe(db: Db, Form(form): Form<FormData>) -> impl IntoResponse {
    if let Ok(new_subscriber) = form.try_into() {
        match insert_subscriber(&db, &new_subscriber).await {
            Ok(_) => http::StatusCode::OK,
            Err(_) => http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    } else {
        http::StatusCode::BAD_REQUEST
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, pool)
)]
pub async fn insert_subscriber(
    pool: &PgPool,
    new_subscriber: &NewSubscriber,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at)
    VALUES ($1, $2, $3, $4)
            "#,
        Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}
