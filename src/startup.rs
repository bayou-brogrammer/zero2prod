use crate::routes;
use axum::{
    http::Request,
    routing::{get, post},
    Router,
};
use surrealdb::{engine::remote::ws::Client, Surreal};
use tokio::{io, net::TcpListener};
use tower::ServiceBuilder;
use tower_http::{
    request_id::{MakeRequestId, RequestId},
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
    ServiceBuilderExt,
};
use tracing::Level;
use uuid::Uuid;

// from https://docs.rs/tower-http/0.2.5/tower_http/request_id/index.html#using-uuids
#[derive(Clone)]
struct MakeRequestUuid;

impl MakeRequestId for MakeRequestUuid {
    fn make_request_id<B>(&mut self, _: &Request<B>) -> Option<RequestId> {
        let request_id = Uuid::new_v4().to_string();

        Some(RequestId::new(request_id.parse().unwrap()))
    }
}

pub async fn run(listener: TcpListener, db: Surreal<Client>) -> io::Result<()> {
    let app = Router::new()
        .route("/health_check", get(routes::health_check))
        .route("/subscriptions", post(routes::subscribe))
        .with_state(db.clone())
        .layer(
            // from https://docs.rs/tower-http/0.2.5/tower_http/request_id/index.html#using-trace
            ServiceBuilder::new()
                .set_x_request_id(MakeRequestUuid)
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(
                            DefaultMakeSpan::new()
                                .include_headers(true)
                                .level(Level::INFO),
                        )
                        .on_response(DefaultOnResponse::new().include_headers(true)),
                )
                .propagate_x_request_id(),
        );

    let addr = listener.local_addr()?;
    println!("Server running on http://{addr}");

    axum::serve(listener, app).await
}
