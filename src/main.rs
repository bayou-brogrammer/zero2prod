use zero2prod::startup::Application;
use zero2prod::{
    configuration::get_configuration,
    telemetry::{get_subscriber, init_subscriber},
};

async fn run() -> hyper::Result<()> {
    let configuration = get_configuration().expect("Failed to read configuration");
    Application::build(configuration).run().await
}

#[tokio::main]
async fn main() -> hyper::Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    run().await
}
