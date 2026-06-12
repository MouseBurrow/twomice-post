#!/usr/bin/env bash
set -euo pipefail

DB_USER="twomice"
DB_PASS="twomice"
DB_HOST="127.0.0.1"
DB_PORT="5432"
TEST_DB="post_test"
TEST_DB_URL="postgresql://${DB_USER}:${DB_PASS}@${DB_HOST}:${DB_PORT}/${TEST_DB}"
MIGRATIONS_DIR="$(cd "$(dirname "$0")" && pwd)/../../db/migrations/post"

# Verify connectivity to the server
if ! PGPASSWORD="$DB_PASS" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d postgres -c "SELECT 1" >/dev/null 2>&1; then
  echo "Cannot connect to Postgres at ${HOST}:${PORT}."
  echo "Start it with: docker compose -f ../../db/compose.yaml up -d post-db"
  exit 1
fi

echo "Setting up test database '${TEST_DB}'..."

# Drop and recreate the test database
PGPASSWORD="$DB_PASS" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d postgres <<SQL
DROP DATABASE IF EXISTS ${TEST_DB};
CREATE DATABASE ${TEST_DB} OWNER ${DB_USER};
SQL

# Make sure pgcrypto is available
echo "  pgcrypto extension"
PGPASSWORD="$DB_PASS" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$TEST_DB" -c "CREATE EXTENSION IF NOT EXISTS pgcrypto;" > /dev/null 2>&1 || true

# Run migrations
echo "Running migrations..."
for f in "$MIGRATIONS_DIR"/*.up.sql; do
  echo "  $(basename "$f")"
  PGPASSWORD="$DB_PASS" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$TEST_DB" -f "$f" > /dev/null
done

echo "Running tests..."

EXIT_CODE=0
for test_bin in api_posts api_comments api_replies api_votes lib; do
  echo ""
  echo "=== $test_bin ==="
  if [ "$test_bin" = "lib" ]; then
    POST_DATABASE_URL="$TEST_DB_URL" PORT="9999" cargo test --lib -- --test-threads=1 2>&1 || EXIT_CODE=$?
  else
    POST_DATABASE_URL="$TEST_DB_URL" PORT="9999" cargo test --test "$test_bin" -- --test-threads=1 2>&1 || EXIT_CODE=$?
  fi
done

# Cleanup test database
echo "Cleaning up..."
PGPASSWORD="$DB_PASS" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d postgres <<SQL
UPDATE pg_database SET datallowconn = false WHERE datname = '${TEST_DB}';
SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '${TEST_DB}';
DROP DATABASE IF EXISTS ${TEST_DB};
SQL

if [ "$EXIT_CODE" -ne 0 ]; then
  echo "Tests failed with exit code $EXIT_CODE"
fi

exit $EXIT_CODE
