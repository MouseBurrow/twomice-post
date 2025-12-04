use sqlx::PgPool;
use utils::db_call;
use utils::errors::AuthError;
use uuid::Uuid;

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
        user_id: Uuid,
        post_content: &str,
        image_url: &str,
    ) -> Result<(String, uuid), AuthError> {
        db_call!(
            pool   = pool,
            query  = sqlx::query_as(r#"SELECT created_at FROM post.create_post($1, $2, $3)"#),
            binds  = [user_id, post_content, image_url],
            errors = {
                "23505" => AuthError::UsernameExists
            }
        )
    }
}
