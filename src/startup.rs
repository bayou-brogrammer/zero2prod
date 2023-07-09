use crate::{
    configuration::{DatabaseSettings, Settings},
    email_client::EmailClient,
    routes,
};
use axum::{
    extract::{FromRef, MatchedPath},
    routing::{get, post},
    Router,
};
use hyper::Request;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    request_id::MakeRequestUuid,
    trace::{DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer},
    ServiceBuilderExt,
};
use tracing::{info_span, Level};

#[derive(Clone, Debug)]
pub struct AppState {
    pub db_pool: PgPool,
    pub base_url: String,
    pub email_client: EmailClient,
}

impl FromRef<AppState> for PgPool {
    fn from_ref(app_state: &AppState) -> PgPool {
        app_state.db_pool.clone()
    }
}

pub struct Application {
    app: Router,
    listener: TcpListener,
}

impl Application {
    pub fn build(settings: Settings) -> Self {
        let db_pool = get_connection_pool(&settings.database);

        let sender_email = settings
            .email_client
            .sender()
            .expect("Invalid sender email address.");

        let timeout = settings.email_client.timeout();
        let email_client = EmailClient::new(
            settings.email_client.base_url,
            sender_email,
            settings.email_client.authorization_token,
            timeout,
        );

        let request_layer = ServiceBuilder::new().layer(
            // from https://docs.rs/tower-http/0.2.5/tower_http/request_id/index.html#using-trace
            ServiceBuilder::new()
                .set_x_request_id(MakeRequestUuid)
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(|request: &Request<_>| {
                            // Log the matched route's path (with placeholders not filled in).
                            // Use request.uri() or OriginalUri if you want the real path.
                            let matched_path = request
                                .extensions()
                                .get::<MatchedPath>()
                                .map(MatchedPath::as_str);

                            info_span!(
                                "http_request",
                                headers = ?request.headers(),
                                method = ?request.method(),
                                matched_path,
                            )
                        })
                        .on_request(DefaultOnRequest::new().level(Level::INFO))
                        .on_response(
                            DefaultOnResponse::new()
                                .include_headers(true)
                                .level(Level::INFO),
                        )
                        .on_failure(
                            DefaultOnFailure::new()
                                .level(Level::INFO)
                                .latency_unit(tower_http::LatencyUnit::Millis),
                        ),
                )
                .propagate_x_request_id(),
        );

        let state = AppState {
            db_pool,
            email_client,
            base_url: settings.application.base_url.clone(),
        };

        let router = Router::new()
            .route("/health_check", get(routes::health_check))
            .route("/subscriptions", post(routes::subscribe))
            .route("/subscriptions/confirm", get(routes::confirm))
            .with_state(state)
            .layer(request_layer);

        let listener = TcpListener::bind(settings.application.address()).unwrap();

        Application {
            app: router,
            listener,
        }
    }

    #[must_use]
    pub fn address(&self) -> String {
        format!("{}", self.listener.local_addr().unwrap())
    }

    #[must_use]
    pub fn port(&self) -> u16 {
        self.listener.local_addr().unwrap().port()
    }

    pub async fn run(self) -> Result<(), hyper::Error> {
        hyper::Server::from_tcp(self.listener)?
            .serve(self.app.into_make_service())
            .await
    }
}

#[must_use]
pub fn get_connection_pool(settings: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(settings.with_db())
}
