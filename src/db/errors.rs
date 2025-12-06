#[derive(Debug)]
pub enum PostError {
    TokenNotFound,
    PostNotFound,
    Db(sqlx::Error),
}
