use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};

/// Get a connection pool from POST_DATABASE_URL.
pub async fn get_db_pool() -> Pool<Postgres> {
    let url = std::env::var("POST_DATABASE_URL").expect(
        "POST_DATABASE_URL must be set. Run via ./run_tests.sh or set it manually.",
    );
    PgPoolOptions::new()
        .max_connections(2)
        .connect(&url)
        .await
        .expect("Failed to connect to test database")
}

/// Clean all test data from tables (idempotent, safe to call before each test).
pub async fn clean_all(pool: &Pool<Postgres>) {
    sqlx::query("DELETE FROM reply_votes").execute(pool).await.ok();
    sqlx::query("DELETE FROM topic_tags").execute(pool).await.ok();
    sqlx::query("DELETE FROM comment_votes").execute(pool).await.ok();
    sqlx::query("DELETE FROM post_votes").execute(pool).await.ok();
    sqlx::query("DELETE FROM replies").execute(pool).await.ok();
    sqlx::query("DELETE FROM comments").execute(pool).await.ok();
    sqlx::query("DELETE FROM posts").execute(pool).await.ok();
    sqlx::query("DELETE FROM topics").execute(pool).await.ok();
}

/// Seed a minimal dataset: a board, a post, and a comment.
/// Cleans existing data first. Returns (topic_name, post_slug, comment_hash).
pub async fn seed_minimal(pool: &Pool<Postgres>) -> (String, String, String) {
    clean_all(pool).await;

    sqlx::query("INSERT INTO topics (name, description) VALUES ($1, $2)")
        .bind("test-board")
        .bind("A test board")
        .execute(pool)
        .await
        .expect("seed topic");

    let topic_id: i64 = sqlx::query_scalar("SELECT id FROM topics WHERE name = $1")
        .bind("test-board")
        .fetch_one(pool)
        .await
        .expect("get topic id");

    let post_id: i64 = sqlx::query_scalar(
        "INSERT INTO posts (creator_id, topic_id, title, slug, content)
         VALUES (100, $1, 'Test Post', '', 'Hello world')
         RETURNING id",
    )
    .bind(topic_id)
    .fetch_one(pool)
    .await
    .expect("seed post");

    let post_slug: String = sqlx::query_scalar(
        "UPDATE posts SET slug = $1 WHERE id = $2 RETURNING slug",
    )
    .bind("test-post")
    .bind(post_id)
    .fetch_one(pool)
    .await
    .expect("set slug");

    let comment_hash: String = sqlx::query_scalar(
        "INSERT INTO comments (hash, sender_id, post_id, content)
         VALUES ('abc12', 101, $1, 'A test comment')
         RETURNING hash",
    )
    .bind(post_id)
    .fetch_one(pool)
    .await
    .expect("seed comment");

    ("test-board".into(), post_slug, comment_hash)
}
