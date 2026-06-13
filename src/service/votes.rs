use super::PostError;
use easy_errors::map_sqlx_error;
use sqlx::{AssertSqlSafe, Pool, Postgres};
use tracing::info;

macro_rules! vote_fn {
    ($name:ident, $table:ident, $id_col:literal) => {
        pub async fn $name(
            pool: &Pool<Postgres>,
            user_id: i64,
            resource_id: i64,
            direction: i8,
        ) -> Result<i64, PostError> {
            let table = stringify!($table);
            if direction == 0 {
                let sql = format!("DELETE FROM {} WHERE user_id = $1 AND {} = $2", table, $id_col);
                sqlx::query(AssertSqlSafe(sql))
                    .bind(user_id)
                    .bind(resource_id)
                    .execute(pool)
                    .await
                    .map_err(map_sqlx_error::<PostError>)?;
            } else {
                let dir: i16 = if direction > 0 { 1 } else { -1 };
                let sql = format!(
                    "INSERT INTO {0} (user_id, {1}, direction) \
                     VALUES ($1, $2, $3) \
                     ON CONFLICT (user_id, {1}) DO UPDATE SET direction = $3",
                    table, $id_col
                );
                sqlx::query(AssertSqlSafe(sql))
                    .bind(user_id)
                    .bind(resource_id)
                    .bind(dir)
                    .execute(pool)
                    .await
                    .map_err(map_sqlx_error::<PostError>)?;
            }

            let count_sql = format!(
                "SELECT COALESCE(SUM(direction), 0)::BIGINT FROM {} WHERE {} = $1",
                table, $id_col
            );
            let count: i64 = sqlx::query_scalar(AssertSqlSafe(count_sql))
                .bind(resource_id)
                .fetch_one(pool)
                .await
                .map_err(map_sqlx_error::<PostError>)?;

            info!(target: "post", "{} vote cast user_id={} resource_id={} direction={} count={}", stringify!($name), user_id, resource_id, direction, count);
            Ok(count)
        }
    };
}

vote_fn!(cast_post_vote, post_votes, "post_id");
vote_fn!(cast_comment_vote, comment_votes, "comment_id");
vote_fn!(cast_reply_vote, reply_votes, "reply_id");
