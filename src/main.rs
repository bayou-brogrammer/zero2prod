use std::net::TcpListener;

use secrecy::ExposeSecret;
use sqlx::postgres::PgPoolOptions;
use zero2prod::startup::run;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration =
        zero2prod::configuration::get_configuration().expect("Failed to read configuration.");
    let connection_pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy(configuration.database.connection_string().expose_secret())
        .expect("Failed to connect to Postgres.");

    let listener = TcpListener::bind(configuration.application.address()).unwrap();
    run(listener, connection_pool).await.unwrap();
}
