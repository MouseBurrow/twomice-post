use crate::db::errors::PostError;
use burrow_db::db_call;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

pub struct CommentHandler {}

impl CommentHandler {
    /// Tries to adda a comment to a post in the database.
    ///
    /// # Arguments
    /// * `pool` - A reference to a database connection pool.
    /// * `token` - active token of the user.
    /// * 'content' - content of the comment
    ///
    ///
    /// # Returns
    /// Time of creation
    ///
    /// # Errors
    /// * `PostError::TokenNotFound` - An error indicating that the the given token can't be found
    /// * `PostError::PostNotFound` - An error indicating that the the given post id can't be found
    /// * `PostError::Db` - If there was an unexpected error!
    pub async fn add_comment(
        pool: &PgPool,
        token: &str,
        comment_content: &str,
        post_id: Uuid,
    ) -> Result<DateTime<Utc>, PostError> {
        db_call!(
            pool = pool,
            query = sqlx::query_scalar(r#"SELECT comment_on_post($1, $2, $3)"#),
            binds = [token, post_id, comment_content],
            error = PostError
        )
    }
}
