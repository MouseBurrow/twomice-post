use std::io::Write;
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

/// If the database in the URL doesn't exist, create it via psql.
fn ensure_database_exists(url: &str) {
    let db_name = url.rsplit('/').next().expect("invalid database URL");
    // Connect to the 'postgres' admin database to create the test DB.
    let admin_url = url
        .rfind('/')
        .map(|i| format!("{}/postgres", &url[..i]))
        .unwrap_or_else(|| "postgresql://twomice:twomice@127.0.0.1:5432/postgres".into());

    let output = Command::new("psql")
        .arg(&admin_url)
        .arg("-c")
        .arg(format!("CREATE DATABASE \"{db_name}\""))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output();
    // Ignore errors — database might already exist.
    drop(output);
}

/// Run all `.up.sql` migrations via psql.
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
        let sql = std::fs::read_to_string(path).expect("failed to read migration file");
        let mut child = Command::new("psql")
            .arg(url)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap_or_else(|_| panic!("psql must be installed. Tried to run: psql {url}"));
        child
            .stdin
            .take()
            .unwrap()
            .write_all(sql.as_bytes())
            .expect("failed to pipe migration to psql");
        let status = child.wait().expect("psql failed");
        if !status.success() {
            panic!("Migration {} failed. Check that the database exists and is empty.", path.display());
        }
    }
}

/// Start a Postgres Docker container, run migrations, return the database URL.
/// The container is scheduled for cleanup on process exit.
async fn start_docker_postgres() -> String {
    let port = pick_free_port().await;
    let container_name = format!("post_test_{port}");

    let output = Command::new("docker")
        .args([
            "run", "-d",
            "--name", &container_name,
            "-e", "POSTGRES_USER=twomice",
            "-e", "POSTGRES_PASSWORD=twomice",
            "-e", "POSTGRES_DB=post",
            "-p", &format!("{port}:5432"),
            "postgres:16",
        ])
        .output()
        .expect("Docker must be installed and running for automatic Postgres setup");
    if !output.status.success() {
        panic!(
            "Failed to start Docker Postgres: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Wait for Postgres to be ready (handles the init restart cycle)
    let wait_name = container_name.clone();
    tokio::task::spawn_blocking(move || {
        std::thread::sleep(std::time::Duration::from_secs(3));
        for _ in 0..30 {
            let ready = Command::new("docker")
                .args(["exec", &wait_name, "pg_isready", "-U", "twomice"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            if ready {
                std::thread::sleep(std::time::Duration::from_secs(2));
                let still_ready = Command::new("docker")
                    .args(["exec", &wait_name, "pg_isready", "-U", "twomice"])
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
    let cleanup_name = container_name.clone();
    tokio::task::spawn_blocking(move || {
        std::thread::sleep(std::time::Duration::from_secs(1));
        let _ = Command::new("docker")
            .args(["rm", "-f", &cleanup_name])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
    });

    format!("postgresql://twomice:twomice@127.0.0.1:{port}/post")
}

async fn pick_free_port() -> u16 {
    tokio::task::spawn_blocking(|| {
        std::net::TcpListener::bind("127.0.0.1:0")
            .and_then(|l| l.local_addr())
            .map(|a| a.port())
            .unwrap_or(15433)
    })
    .await
    .unwrap()
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
