use crate::routes;
use axum::{
    routing::{get, post},
    Router,
};
use surrealdb::{engine::remote::ws::Client, Surreal};
use tokio::{io, net::TcpListener};

pub async fn run(listener: TcpListener, db: Surreal<Client>) -> io::Result<()> {
    let app = Router::new()
        .route("/health_check", get(routes::health_check))
        .route("/subscriptions", post(routes::subscribe))
        .with_state(db.clone());

    let addr = listener.local_addr()?;
    println!("Server running on http://{addr}");

    axum::serve(listener, app).await
}
