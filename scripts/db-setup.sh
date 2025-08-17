#!/usr/bin/env bash
# scripts/db-setup.sh
# Apply DB grants needed by the ecoblock app. Safe to run multiple times.

set -euo pipefail
PSQL_URL=${DATABASE_URL:-postgres://postgres:postgres@localhost:5432/ecoblock}
APP_ROLE=${APP_DB_ROLE:-ecoblock}

echo "Using DB URL: ${PSQL_URL}" >&2
echo "Granting privileges to role: ${APP_ROLE}" >&2

psql "$PSQL_URL" -c "GRANT SELECT, INSERT, UPDATE, DELETE ON TABLE public.blogs TO ${APP_ROLE};"
psql "$PSQL_URL" -c "GRANT SELECT, INSERT, UPDATE, DELETE ON TABLE public.stories TO ${APP_ROLE};"
psql "$PSQL_URL" -c "GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO ${APP_ROLE};"

echo "Done."
