use crate::configuration::Db;
use axum::{http, response::IntoResponse, Form};
use chrono::{DateTime, Utc};
use surrealdb::sql::Thing;

// const SUBSCRIPTIONS: &str = "subscriptions";

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
    let request_id = uuid::Uuid::new_v4();
    let request_span = tracing::info_span!(
        "Adding a new subscriber",
        %request_id,
        email = %form.email,
        name = %form.name
    );

    let _guard = request_span.enter();

    match db
        .query(
            "CREATE subscriptions:uuid() SET name = $name, email = $email, subscribed_at = time::now();",
        )
        .bind(("name", form.name))
        .bind(("email", form.email))
        .await
    {
        Ok(_) => {
            tracing::info!("Request Id {request_id} - Successfully saved subscriber details");
            http::StatusCode::OK
        },
        Err(e) => {
            tracing::error!("Request id {request_id} - Error saving subscriber details: {:?}", e);
            http::StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
