mod routes;

use axum::{
    routing::{get, post},
    Router,
};
use tokio::{io, net::TcpListener};

pub async fn run(listener: TcpListener) -> io::Result<()> {
    let app = Router::new()
        .route("/health_check", get(routes::health_check))
        .route("/subscriptions", post(routes::subscribe));

    let addr = listener.local_addr()?;
    println!("Server running on http://{}:{}", addr.ip(), addr.port());

    axum::serve(listener, app).await
}
