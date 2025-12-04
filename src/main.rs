use crate::db::post::PostHandler;
use config::Config;
use sqlx::postgres::PgPoolOptions;

mod db;
mod models;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let service_name = "POST";

    let config: Config = Config::init(service_name);

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(config.database_url.as_str())
        .await?;

    // TEST

    let user_id: uuid = "a1538a75-97a0-4644-859f-d41d7615182d";
    let content = "contet";
    let image_url = "image.png";
    match PostHandler::create_post(&pool, user_id, content, image_url).await {
        Ok((creation_date, post_id)) => {
            println!("Post successfully created!");
            println!("created at: {}", user_id);
            println!("post_id: {}", post_id);
        }
        Err(e) => match e {
            AuthError::InvalidPassword => {
                println!("Invalid password");
            }
            AuthError::UserNotFound => {
                println!("User with that name not found, creating a new account");

                match AuthHandler::create_account(&pool, username, password_hash.as_str()).await {
                    Ok((user_id, token)) => {
                        println!("Account created successfully!");
                        println!("User ID : {}", user_id);
                        println!("Token   : {}", token);
                    }
                    Err(e) => match e {
                        AuthError::UsernameExists => {
                            println!("Account already exists!");
                        }
                        AuthError::Db(err) => {
                            println!("Unexpected database error: {}", err);
                        }
                        _ => (),
                    },
                }
            }
            AuthError::Db(err) => {
                println!("Unexpected database error: {}", err);
            }
            _ => (),
        },
    }

    Ok(())
}
