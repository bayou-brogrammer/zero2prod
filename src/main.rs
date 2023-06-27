use tokio::{io, net::TcpListener};
use zero2prod::startup::run;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> io::Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration =
        zero2prod::configuration::get_configuration().expect("Failed to read configuration.");

    let db = zero2prod::db::connect(&configuration.database).await;

    let listener = TcpListener::bind(format!("127.0.0.1:{}", configuration.application_port))
        .await
        .unwrap();

    run(listener, db).await
}
