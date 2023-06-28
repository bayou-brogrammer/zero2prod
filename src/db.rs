use std::net::SocketAddr;

use secrecy::ExposeSecret;
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    Surreal,
};

use crate::configuration::{DatabaseSettings, Settings};

pub struct TestApp {
    pub addr: SocketAddr,
    pub config: Settings,
}

pub async fn connect(config: &DatabaseSettings) -> Surreal<Client> {
    let connection_string = config.connection_string();

    // Setup surrealdb connection
    let db = Surreal::new::<Ws>(connection_string.expose_secret().as_str())
        .await
        .unwrap();

    if let Err(err) = db
        .signin(Root {
            username: &config.username,
            password: config.password.expose_secret(),
        })
        .await
    {
        panic!("Failed to sign in as root: {err}");
    }

    db.use_ns(config.namespace.clone())
        .use_db(config.name.clone())
        .await
        .unwrap();

    db
}
