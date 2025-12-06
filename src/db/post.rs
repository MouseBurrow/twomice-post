use crate::db::errors::PostError;
use burrow_db::db_call;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

pub struct PostHandler {}

impl PostHandler {
    /// Tries to create a post in the database.
    ///
    /// # Arguments
    /// * `pool` - A reference to a database connection pool.
    /// * `username` - The username of the new account.
    /// * 'content' - content of the post
    /// * 'image url' - url of the post image
    ///
    ///
    /// # Returns
    /// A tuple containing the post ID, user id, content, url and creation date.
    ///
    /// # Errors
    /// * `AuthError::UsernameExists` - An error indicating that the account with the given username already exists.
    /// * `AuthError::Db` - If there was an unexpected error!
    pub async fn create_post(
        pool: &PgPool,
        token: &str,
        post_content: &str,
        image_url: &str,
    ) -> Result<DateTime<Utc>, PostError> {
        db_call!(
            pool   = pool,
            query  = sqlx::query_scalar(r#"SELECT create_post($1, $2, $3)"#),
            binds  = [token, post_content, image_url],
            errors = {
                "CP000" => PostError::TokenNotFound
            },
            fallback = PostError::Db
        )
    }
}
