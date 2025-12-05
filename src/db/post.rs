use crate::db::errors::PostError;
use burrow_db::db_call;
use sqlx::PgPool;
use uuid::Uuid;

pub struct PostHandler {}

impl PostHandler {
    /// Tries to create a post in the database.
    ///
    /// # Arguments
    /// * `pool` - A reference to a database connection pool.
    /// * `username` - The username of the new account.
    /// * 'content' - Content of the post
    /// * 'image url' - Url of the post image
    ///
    ///
    /// # Returns
    /// A tuple containing the post ID and creation date.
    ///
    /// # Errors
    /// * `AuthError::Db` - If there was an unexpected error!
    pub async fn create_post(
        pool: &PgPool,
        user_id: Uuid,
        post_content: &str,
        image_url: &str,
    ) -> Result<(Uuid, String), PostError> {
        db_call!(
            pool = pool,
            query = sqlx::query_as(r#"SELECT created_at FROM post.create_post($1, $2, $3)"#),
            binds = [user_id, post_content, image_url],
            fallback = PostError::Db
        )
    }
}
