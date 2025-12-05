#[derive(Debug)]
pub enum PostError {
    UserNotFound,
    Db(sqlx::Error),
}
