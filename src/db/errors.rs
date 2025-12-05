#[derive(Debug)]
pub enum PostError {
    TokenNotFound,
    Db(sqlx::Error),
}
