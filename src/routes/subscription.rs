use axum::{http, response::IntoResponse, Form};

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(Form(form): Form<FormData>) -> impl IntoResponse {
    http::StatusCode::OK
}
