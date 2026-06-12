use std::path::Path;
use std::process::{Command, Stdio};

use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};

/// Get a connection pool to a test Postgres database.
///
/// Priority:
/// 1. `POST_DATABASE_URL` env var — connect directly, run migrations
/// 2. No env var — start a Docker container, run migrations, connect
///
/// Requires `psql` on PATH for both migration paths.
/// Requires Docker for the fallback path.
pub async fn get_db_pool() -> Pool<Postgres> {
    let url = if let Ok(url) = std::env::var("POST_DATABASE_URL") {
        url
    } else {
        start_docker_postgres().await
    };

    ensure_database_exists(&url);
    run_migrations_psql(&url);

    PgPoolOptions::new()
        .max_connections(2)
        .connect(&url)
        .await
        .expect("failed to connect to test database after setup")
}

/// Drop and recreate the test database, then run migrations.
/// Only drops databases named `post_test*` to avoid accidental data loss.
fn ensure_database_exists(url: &str) {
    let db_name = url.rsplit('/').next().expect("invalid database URL");

    let admin_url = url
        .rfind('/')
        .map(|i| format!("{}/postgres", &url[..i]))
        .unwrap_or_else(|| "postgresql://twomice:twomice@127.0.0.1:5432/postgres".into());

    // Only drop/recreate dedicated test databases, not the main "post" db.
    if db_name.starts_with("post_test") {
        Command::new("psql")
            .arg(&admin_url)
            .arg("-c")
            .arg(&format!("DROP DATABASE IF EXISTS \"{db_name}\""))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .expect("psql failed");

        Command::new("psql")
            .arg(&admin_url)
            .arg("-c")
            .arg(&format!("CREATE DATABASE \"{db_name}\""))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .expect("psql failed");
    }
}

/// Run all `.up.sql` migrations via psql.
/// Errors (like "already exists") are expected when re-running against an existing DB.
fn run_migrations_psql(url: &str) {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let migrations_dir = Path::new(manifest).join("../../db/migrations/post");

    let mut entries: Vec<_> = std::fs::read_dir(&migrations_dir)
        .expect("migrations directory not found (run from services/post/)")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension().map_or(false, |e| e == "sql")
                && p.to_string_lossy().ends_with(".up.sql")
        })
        .collect();
    entries.sort();

    for path in &entries {
        Command::new("psql")
            .arg(url)
            .arg("-f")
            .arg(path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .unwrap_or_else(|_| panic!("psql must be installed. Tried to run: psql {url}"));
        // Exit code is ignored — "already exists" errors are expected
        // when running migrations against an existing database.
    }
}

/// Start a Postgres Docker container, run migrations, return the database URL.
/// The container is scheduled for cleanup on process exit.
async fn start_docker_postgres() -> String {
    // First, check if there's already a Postgres on localhost:5432.
    let existing = try_connect_existing().await;
    if let Some(url) = existing {
        return url;
    }

    // Try starting a new container with host networking (avoids docker-proxy issues).
    let container_name = "post_test_auto";
    let url = "postgresql://twomice:twomice@127.0.0.1:5432/post".to_string();

    // Clean up any leftover container from a previous run.
    let _ = Command::new("docker")
        .args(["rm", "-f", container_name])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    let output = Command::new("docker")
        .args([
            "run", "-d",
            "--name", container_name,
            "--network=host",
            "-e", "POSTGRES_USER=twomice",
            "-e", "POSTGRES_PASSWORD=twomice",
            "-e", "POSTGRES_DB=post",
            "postgres:16",
        ])
        .output()
        .expect("Docker must be installed and running for automatic Postgres setup");
    if !output.status.success() {
        panic!(
            "Failed to start Docker Postgres.\n\
             Set POST_DATABASE_URL to point to an existing Postgres, or \
             start one manually:\n  \
             docker run -d --name post_test -p 5433:5432 -e POSTGRES_USER=twomice \
             -e POSTGRES_PASSWORD=twomice -e POSTGRES_DB=post postgres:16\n\
             Export: POST_DATABASE_URL=postgresql://twomice:twomice@127.0.0.1:5433/post\n\
             Docker error: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Wait for Postgres to be ready (handles the init restart cycle)
    let container = container_name.to_string();
    tokio::task::spawn_blocking(move || {
        std::thread::sleep(std::time::Duration::from_secs(3));
        for _ in 0..30 {
            let ready = Command::new("docker")
                .args(["exec", &container, "pg_isready", "-U", "twomice"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            if ready {
                std::thread::sleep(std::time::Duration::from_secs(2));
                let still_ready = Command::new("docker")
                    .args(["exec", &container, "pg_isready", "-U", "twomice"])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false);
                if still_ready {
                    return;
                }
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
        panic!("Timed out waiting for Postgres container to be ready");
    })
    .await
    .unwrap();

    // Schedule cleanup
    let cleanup = container_name.to_string();
    tokio::task::spawn_blocking(move || {
        std::thread::sleep(std::time::Duration::from_secs(1));
        let _ = Command::new("docker")
            .args(["rm", "-f", &cleanup])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
    });

    url
}

/// Try to connect to an existing Postgres on localhost:5432.
/// If it responds, we use it rather than starting Docker.
async fn try_connect_existing() -> Option<String> {
    let url = "postgresql://twomice:twomice@127.0.0.1:5432/post".to_string();
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await
        .ok()?;
    drop(pool);
    Some(url)
}

/// Delete all data from all tables (safe ordering for FK constraints).
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
