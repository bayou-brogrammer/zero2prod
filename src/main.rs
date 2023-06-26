use surrealdb::Surreal;
use surrealdb::{engine::remote::ws::Ws, opt::auth::Root};
use tokio::{io, net::TcpListener};
use zero2prod::startup::run;

#[tokio::main]
async fn main() -> io::Result<()> {
    let configuration =
        zero2prod::configuration::get_configuration().expect("Failed to read configuration.");

    let db_config = configuration.database;
    let connection_string = db_config.connection_string();
    let namespace = db_config.namespace;
    let db_name = db_config.name;

    let db = Surreal::new::<Ws>(connection_string).await.unwrap();

    if let Err(err) = db
        .signin(Root {
            username: "root",
            password: "root",
        })
        .await
    {
        panic!("Failed to sign in as root: {}", err);
    }

    db.use_ns(namespace).use_db(db_name).await.unwrap();

    let listener = TcpListener::bind(format!("127.0.0.1:{}", configuration.application_port))
        .await
        .unwrap();

    run(listener, db).await
}
