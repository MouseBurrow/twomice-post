use config::Config;
use sqlx::postgres::PgPoolOptions;

mod db;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let service_name = "POST";

    let config: Config = Config::init(service_name);
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(config.database_url.as_str())
        .await?;

    Ok(())
}
