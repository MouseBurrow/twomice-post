#!/usr/bin/env bash
# Helper script: runs tests with a dedicated Postgres database.
# Tests run sequentially to avoid DB race conditions.
#
# Usage: ./run_tests.sh
set -euo pipefail

# Only set up if not already configured
if [ -z "${POST_DATABASE_URL:-}" ]; then
  DB_USER="${DB_USER:-twomice}"
  DB_PASS="${DB_PASS:-twomice}"
  DB_NAME="${DB_NAME:-post}"
  DB_HOST="${DB_HOST:-127.0.0.1}"
  DB_PORT="${DB_PORT:-5432}"
  TEST_DB="${TEST_DB:-post_test}"
  TEST_DB_URL="postgresql://${DB_USER}:${DB_PASS}@${DB_HOST}:${DB_PORT}/${TEST_DB}"
  MIGRATIONS_DIR="$(cd "$(dirname "$0")" && pwd)/../../db/migrations/post"

  # Check if the Postgres server is reachable
  if PGPASSWORD="$DB_PASS" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d postgres -c "SELECT 1" >/dev/null 2>&1; then
    echo "Setting up test database '${TEST_DB}'..."
    PGPASSWORD="$DB_PASS" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d postgres <<SQL
DROP DATABASE IF EXISTS ${TEST_DB};
CREATE DATABASE ${TEST_DB} OWNER ${DB_USER};
SQL
    echo "Running migrations..."
    for f in "$MIGRATIONS_DIR"/*.up.sql; do
      echo "  $(basename "$f")"
      PGPASSWORD="$DB_PASS" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$TEST_DB" -f "$f" > /dev/null
    done
    export POST_DATABASE_URL="$TEST_DB_URL"
  else
    echo "No Postgres reachable at ${DB_HOST}:${DB_PORT}."
    echo "The test code will start a Docker container automatically."
    echo ""
  fi
fi

# Run each test target sequentially (avoids DB race conditions between binaries)
EXIT_CODE=0
for target in lib api_comments api_posts api_replies api_votes; do
  echo ""
  echo "=== $target ==="
  if [ "$target" = "lib" ]; then
    cargo test --lib -- --test-threads=1 2>&1 || EXIT_CODE=$?
  else
    cargo test --test "$target" -- --test-threads=1 2>&1 || EXIT_CODE=$?
  fi
done

exit $EXIT_CODE
