use crate::configuration::Db;
use axum::{http, response::IntoResponse, Form};
use chrono::{DateTime, Utc};
use surrealdb::sql::Thing;

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct FormData {
    email: String,
    name: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Subscription {
    pub id: Thing,
    pub name: String,
    pub email: String,
    pub subscribed_at: DateTime<Utc>,
}

#[allow(clippy::async_yields_async)]
#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, db),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(db: Db, Form(form): Form<FormData>) -> impl IntoResponse {
    match insert_subscriber(db, form).await {
        Ok(_) => http::StatusCode::OK,
        Err(_) => http::StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[tracing::instrument(name = "Saving new subscriber details in the database", skip(form, db))]
pub async fn insert_subscriber(db: Db, form: FormData) -> Result<(), surrealdb::Error> {
    db
      .query(
          "CREATE subscriptions:uuid() SET name = $name, email = $email, subscribed_at = time::now();",
      )
      .bind(("name", form.name))
      .bind(("email", form.email))
      .await.map_err(|e| {
          tracing::error!("Failed to execute query: {:?}", e);
          e
      })?;

    Ok(())
}
