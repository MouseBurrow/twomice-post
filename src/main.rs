use actix_web::{App, HttpServer};
use config::Config;
use sqlx::postgres::PgPoolOptions;
use std::env;

mod db;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // If it is dev mode, load env variables from .env
    #[cfg(debug_assertions)]
    Config::load_local_env();

    // Connect to the database
    let config: Config = Config::init("POST");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(config.database_url.as_str())
        .await?;

    // Start listening endpoint
    let port = env::var("PORT")?;
    let addr = format!("0.0.0.0:{}", port);
    println!("Starting Gateway at http://{}", addr);
    HttpServer::new(App::new)
        .bind(addr)?
        .run()
        .await
        .expect("Could not connect to network");

    Ok(())
}
