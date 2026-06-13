#!/usr/bin/env bash
# Runs tests against a temporary Postgres container.
# Handles container lifecycle (start → wait → migrate → test → cleanup).
#
# Usage: ./run_tests.sh
set -euo pipefail

CONTAINER_NAME="twomice-post-test"
DB_USER="twomice"
DB_PASS="twomice"
DB_NAME="post"
MIGRATIONS_DIR="$(dirname "$0")/migrations"

cleanup() {
  echo "Cleaning up..."
  docker rm -f "$CONTAINER_NAME" >/dev/null 2>&1 || true
}
trap cleanup EXIT

DB_URL=""

# First, check if there's already a Postgres running (e.g. from docker-compose)
if PGPASSWORD="$DB_PASS" psql -h 127.0.0.1 -p 5432 -U "$DB_USER" -d postgres -c "SELECT 1" >/dev/null 2>&1; then
  echo "Using existing Postgres at 127.0.0.1:5432"
  # Create a dedicated test database
  PGPASSWORD="$DB_PASS" psql -h 127.0.0.1 -p 5432 -U "$DB_USER" -d postgres \
    -c "DROP DATABASE IF EXISTS ${DB_NAME}_test" \
    -c "CREATE DATABASE ${DB_NAME}_test OWNER ${DB_USER}" > /dev/null
  # Run migrations
  echo "Running migrations..."
  for f in "$MIGRATIONS_DIR"/*.up.sql; do
    PGPASSWORD="$DB_PASS" psql -h 127.0.0.1 -p 5432 -U "$DB_USER" -d "${DB_NAME}_test" -f "$f" > /dev/null 2>&1 || true
  done
  DB_URL="postgresql://${DB_USER}:${DB_PASS}@127.0.0.1:5432/${DB_NAME}_test"

elif docker ps --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
  echo "Reusing existing container $CONTAINER_NAME"
  DB_URL="postgresql://${DB_USER}:${DB_PASS}@127.0.0.1:5432/${DB_NAME}"

else
  echo "Starting Postgres container..."
  # Use bridge networking with a unique port to avoid conflicts
  docker rm -f "$CONTAINER_NAME" >/dev/null 2>&1 || true
  docker run -d \
    --name "$CONTAINER_NAME" \
    -p 5432 \
    -e POSTGRES_USER="$DB_USER" \
    -e POSTGRES_PASSWORD="$DB_PASS" \
    -e POSTGRES_DB="$DB_NAME" \
    postgres:16 > /dev/null

  # Get the mapped port
  HOST_PORT=$(docker port "$CONTAINER_NAME" 5432 | head -1 | sed 's/.*://')

  # Wait for Postgres (handles the init restart cycle)
  for i in $(seq 1 30); do
    if docker exec "$CONTAINER_NAME" pg_isready -U "$DB_USER" -d "$DB_NAME" >/dev/null 2>&1; then
      sleep 2
      if docker exec "$CONTAINER_NAME" pg_isready -U "$DB_USER" -d "$DB_NAME" >/dev/null 2>&1; then
        break
      fi
    fi
    if [ "$i" -eq 30 ]; then
      echo "Timed out waiting for Postgres"
      exit 1
    fi
    sleep 1
  done

  # Run migrations
  echo "Running migrations..."
  for f in "$MIGRATIONS_DIR"/*.up.sql; do
    docker exec -i "$CONTAINER_NAME" psql -U "$DB_USER" -d "$DB_NAME" < "$f" > /dev/null 2>&1 || true
  done

  DB_URL="postgresql://${DB_USER}:${DB_PASS}@127.0.0.1:${HOST_PORT}/${DB_NAME}"
fi

export POST_DATABASE_URL="$DB_URL"
echo "POST_DATABASE_URL=$DB_URL"

# Compile once, run each binary sequentially
echo "Compiling tests..."
cargo test --no-run 2>&1

echo ""
EXIT_CODE=0
for target in lib api_comments api_posts api_replies api_votes; do
  echo "=== $target ==="
  if [ "$target" = "lib" ]; then
    cargo test --lib -- --test-threads=1 2>&1 || EXIT_CODE=$?
  else
    cargo test --test "$target" -- --test-threads=1 2>&1 || EXIT_CODE=$?
  fi
done

exit $EXIT_CODE
