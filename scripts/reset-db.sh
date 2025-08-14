#!/usr/bin/env bash
set -euo pipefail

# Usage: scripts/reset-db.sh
# Drops and recreates the ecoblock DB and runs migrations via the app.

DB_URL=${DATABASE_URL:-postgres://ecoblock:ecopass@localhost:5432/ecoblock}

# parse database name from URL (simple)
DB_NAME=$(echo "$DB_URL" | sed -E 's|.*/([^/?]+)(\?.*)?$|\1|')
DB_USER=$(echo "$DB_URL" | sed -E 's|postgres://([^:]+):.*@.*|\1|')

echo "Dropping and recreating DB $DB_NAME owned by $DB_USER"
dropdb "$DB_NAME" || true
createdb -O "$DB_USER" "$DB_NAME"
psql -d "$DB_NAME" -c "CREATE EXTENSION IF NOT EXISTS pgcrypto;"

echo "Running app to apply migrations (cargo run)"
cargo run
