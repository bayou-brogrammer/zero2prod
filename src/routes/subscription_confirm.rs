use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use hyper::http;
use sqlx::PgPool;
use uuid::Uuid;

use crate::startup::AppState;

#[derive(serde::Deserialize, Debug)]
pub struct Parameters {
    subscription_token: String,
}

#[allow(clippy::async_yields_async)]
#[tracing::instrument(name = "Confirm a pending subscriber", skip(params, state))]
pub async fn confirm(
    State(state): State<AppState>,
    Query(params): Query<Parameters>,
) -> impl IntoResponse {
    let Ok(id) = get_subscriber_id_from_token(&state.db_pool, &params.subscription_token).await
    else {
        return http::StatusCode::NOT_FOUND;
    };

    match id {
        Some(subscriber_id) => match confirm_subscriber(&state.db_pool, subscriber_id).await {
            Ok(_) => http::StatusCode::OK,
            Err(_) => http::StatusCode::INTERNAL_SERVER_ERROR,
        },
        None => http::StatusCode::UNAUTHORIZED,
    }
}

#[allow(clippy::async_yields_async)]
#[tracing::instrument(name = "Mark subscriber as confirmed", skip(subscriber_id, pool))]
async fn confirm_subscriber(pool: &PgPool, subscriber_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1"#,
        subscriber_id
    )
    .execute(pool)
    .await
    .map(|_| ())
}

#[tracing::instrument(name = "Get subscriber_id from token", skip(subscription_token, pool))]
async fn get_subscriber_id_from_token(
    pool: &PgPool,
    subscription_token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    sqlx::query!(
        r#"SELECT subscriber_id FROM subscription_tokens WHERE subscription_token = $1"#,
        subscription_token
    )
    .fetch_optional(pool)
    .await
    .map(|record| record.map(|r| r.subscriber_id))
}
