use std::path::Path;
use std::process::{Command, Stdio};

use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};

/// Get a connection pool to a test Postgres database.
///
/// 1. `POST_DATABASE_URL` env var — connect to that database, run migrations
/// 2. No env var — start a Docker container, run migrations, connect
pub async fn get_db_pool() -> Pool<Postgres> {
    if let Ok(url) = std::env::var("POST_DATABASE_URL") {
        run_migrations_psql(&url);
        return PgPoolOptions::new()
            .max_connections(2)
            .connect(&url)
            .await
            .unwrap_or_else(|e| panic!("Cannot connect to POST_DATABASE_URL={url}: {e}"));
    }

    let url = start_docker_postgres().await;
    PgPoolOptions::new()
        .max_connections(2)
        .connect(&url)
        .await
        .expect("failed to connect to test database after Docker setup")
}

/// Path to the migration files, relative to the crate root.
fn migrations_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations")
}

/// Run all `.up.sql` migrations via `psql` with a connection URL.
fn run_migrations_psql(url: &str) {
    run_migrations(|sql| {
        let mut child = Command::new("psql")
            .arg(url)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("psql must be installed");
        use std::io::Write;
        child.stdin.take().unwrap().write_all(sql.as_bytes()).ok();
        child.wait().ok();
    });
}

/// Run all `.up.sql` migrations via `docker exec`.
fn run_migrations_docker(container: &str) {
    run_migrations(|sql| {
        let mut child = Command::new("docker")
            .args(["exec", "-i", container, "psql", "-U", "twomice", "-d", "post"])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("docker must be installed");
        use std::io::Write;
        child.stdin.take().unwrap().write_all(sql.as_bytes()).ok();
        child.wait().ok();
    });
}

/// Read all `.up.sql` migration files and run them through the provided executor.
fn run_migrations(mut exec: impl FnMut(&str)) {
    let dir = migrations_dir();
    let mut entries: Vec<_> = std::fs::read_dir(&dir)
        .expect("migrations directory not found")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |e| e == "sql") && p.to_string_lossy().ends_with(".up.sql"))
        .collect();
    entries.sort();
    for path in &entries {
        let sql = std::fs::read_to_string(path).expect("failed to read migration file");
        exec(&sql);
    }
}

/// Start a Postgres Docker container, run migrations, return the database URL.
/// Cleans up the container on process exit (best-effort).
async fn start_docker_postgres() -> String {
    // First, check if there's already a Postgres on localhost:5432.
    let existing = try_connect_existing().await;
    if let Some(url) = existing {
        return url;
    }

    let pid = std::process::id();
    let container_name = format!("post_test_{pid}");

    // Clean any leftover container with this name.
    let _ = Command::new("docker")
        .args(["rm", "-f", &container_name])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    let output = Command::new("docker")
        .args([
            "run",
            "-d",
            "--name",
            &container_name,
            "--network=host",
            "-e",
            "POSTGRES_USER=twomice",
            "-e",
            "POSTGRES_PASSWORD=twomice",
            "-e",
            "POSTGRES_DB=post",
            "postgres:16",
        ])
        .output()
        .expect("Docker must be installed for automatic Postgres setup");
    if !output.status.success() {
        panic!(
            "Failed to start Docker Postgres.\n\
             Set POST_DATABASE_URL or start manually:\n  \
             docker run -d --name post_test -p 5433:5432 -e POSTGRES_USER=twomice \
             -e POSTGRES_PASSWORD=twomice -e POSTGRES_DB=post postgres:16\n\
             Then: POST_DATABASE_URL=postgresql://twomice:twomice@127.0.0.1:5433/post"
        );
    }

    // Wait for Postgres to be ready (handles the init restart cycle)
    let wait = container_name.clone();
    tokio::task::spawn_blocking(move || {
        std::thread::sleep(std::time::Duration::from_secs(3));
        for _ in 0..30 {
            let ready = Command::new("docker")
                .args(["exec", &wait, "pg_isready", "-U", "twomice"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            if ready {
                std::thread::sleep(std::time::Duration::from_secs(2));
                let still = Command::new("docker")
                    .args(["exec", &wait, "pg_isready", "-U", "twomice"])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false);
                if still {
                    return;
                }
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
        panic!("Timed out waiting for Postgres container");
    })
    .await
    .unwrap();

    // Run migrations from inside the container
    run_migrations_docker(&container_name);

    // Schedule cleanup
    let cleanup = container_name.clone();
    tokio::task::spawn_blocking(move || {
        std::thread::sleep(std::time::Duration::from_secs(2));
        let _ = Command::new("docker")
            .args(["rm", "-f", &cleanup])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    });

    "postgresql://twomice:twomice@127.0.0.1:5432/post".to_string()
}

/// Try to connect to an existing Postgres on common hostnames.
async fn try_connect_existing() -> Option<String> {
    for host in &["127.0.0.1:5432", "localhost:5432", "postgres:5432"] {
        let url = format!("postgresql://twomice:twomice@{host}/post");
        if PgPoolOptions::new()
            .max_connections(1)
            .connect(&url)
            .await
            .is_ok()
        {
            return Some(url);
        }
    }
    None
}

/// Delete all data from all tables (safe ordering for FK constraints).
pub async fn clean_all(pool: &Pool<Postgres>) {
    sqlx::query("DELETE FROM reply_votes")
        .execute(pool)
        .await
        .ok();
    sqlx::query("DELETE FROM topic_tags")
        .execute(pool)
        .await
        .ok();
    sqlx::query("DELETE FROM comment_votes")
        .execute(pool)
        .await
        .ok();
    sqlx::query("DELETE FROM post_votes")
        .execute(pool)
        .await
        .ok();
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

    let post_slug: String =
        sqlx::query_scalar("UPDATE posts SET slug = $1 WHERE id = $2 RETURNING slug")
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
