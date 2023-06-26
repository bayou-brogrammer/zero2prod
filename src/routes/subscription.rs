use crate::configuration::Db;
use axum::{http, response::IntoResponse, Form};
use chrono::{DateTime, Utc};
use surrealdb::sql::Thing;

// const SUBSCRIPTIONS: &str = "subscriptions";

#[derive(serde::Deserialize, serde::Serialize)]
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

pub async fn subscribe(db: Db, Form(form): Form<FormData>) -> impl IntoResponse {
    match db
        .query(
            "CREATE subscriptions:uuid() SET name = $name, email = $email, subscribed_at = time::now();",
        )
        .bind(("name", form.name))
        .bind(("email", form.email))
        .await
    {
        Ok(_) => http::StatusCode::OK,
        Err(e) => {
            eprintln!("Error: {e}");
            http::StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
