#!/usr/bin/env bash
set -euo pipefail
mkdir -p data/dumps
TS=$(date +%Y%m%d-%H%M%S)
OUT="data/dumps/ecoblock-${TS}.sql"
if docker ps --format '{{.Names}}' | grep -q "ecoblock-api-kernel_db_\|db"; then
  echo "Detected running docker container; using docker exec on service 'db' if available"
  if command -v docker-compose >/dev/null 2>&1 && docker-compose ps db >/dev/null 2>&1; then
  docker-compose exec -T db pg_dump -U postgres -d ecoblock -F p > "$OUT"
  echo "Dump written to $OUT"
  else
    CONTAINER=$(docker ps --format '{{.Names}}' | grep db | head -n1)
    if [ -z "$CONTAINER" ]; then
      echo "No running db container found" >&2
      exit 1
    fi
    docker exec -i "$CONTAINER" pg_dump -U postgres -d ecoblock -F p > "$OUT"
    echo "Dump written to $OUT"
  fi
else
  echo "No running docker postgres detected; running a temporary pg_dump container"
    docker run --rm -v "$(pwd)/data/dumps:/dumps" postgres:15 \
      sh -c "pg_dump -h host.docker.internal -U postgres -d ecoblock -F p > /dumps/$(basename '$OUT')"
    echo "Dump written to $OUT"
fi
