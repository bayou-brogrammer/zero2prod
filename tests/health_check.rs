use std::net::SocketAddr;

use hyper::StatusCode;

use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    sql::Uuid,
    Surreal,
};
use tokio::net::TcpListener;
use zero2prod::{
    configuration::{get_configuration, DatabaseSettings, Settings},
    routes::Subscription,
    startup::run,
};

pub struct TestApp {
    pub addr: SocketAddr,
    pub config: Settings,
}

async fn connect_db(config: &DatabaseSettings) -> Surreal<Client> {
    let connection_string = config.connection_string();

    // Setup surrealdb connection
    let db = Surreal::new::<Ws>(connection_string).await.unwrap();

    db.signin(Root {
        username: "root",
        password: "root",
    })
    .await
    .unwrap();

    db.use_ns(config.namespace.clone())
        .use_db(config.name.clone())
        .await
        .unwrap();

    db
}

async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind random port");
    let addr = listener.local_addr().unwrap();

    let mut config = get_configuration().expect("Failed to read configuration.");
    config.database.name = Uuid::new_v4().to_string();

    let db = connect_db(&config.database).await;
    tokio::spawn(async move { run(listener, db).await.expect("Failed to bind address") });
    TestApp { addr, config }
}

#[tokio::test]
async fn health_check_works() {
    let TestApp { addr, .. } = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("http://{addr}/health_check"))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let TestApp { addr, config } = spawn_app().await;
    let client = reqwest::Client::new();

    // Setup surrealdb connection
    let db = connect_db(&config.database).await;

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(format!("http://{addr}/subscriptions"))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(200, response.status().as_u16());

    let results: Vec<Subscription> = db
        .select("subscriptions")
        .await
        .expect("Failed to execute query");

    assert!(results.len() == 1);

    let saved = results.first().unwrap();
    assert_eq!(saved.name, "le guin");
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let TestApp { addr, .. } = spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(format!("http://{addr}/subscriptions"))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        // Assert
        assert_eq!(
            StatusCode::UNPROCESSABLE_ENTITY,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}
