SHELL := /bin/bash
PKG := ecoblock-api-kernel

.PHONY: all build migrate db-setup test strip-comments restore-baks

all: build

build:
	cargo build

migrate:
	@echo "Running sqlx migrations..."
	@export DATABASE_URL=${DATABASE_URL:-postgres://postgres:postgres@localhost:5432/ecoblock} ; \
	sqlx migrate run

db-setup:
	@echo "Running DB setup script..."
	@APP_DB_ROLE=${APP_DB_ROLE:-ecoblock} ./scripts/db-setup.sh

test:
	@echo "Run migrations, DB setup and smoke-test"
	@$(MAKE) migrate
	@$(MAKE) db-setup
	@export JWT_SECRET=$$(openssl rand -hex 32) ; \
	JWT_SECRET=$$JWT_SECRET ./scripts/test_endpoints.sh

strip-comments:
	@echo "Stripping comments from Rust sources (creates .bak files)"
	@python3 ./scripts/strip_comments.py src/

restore-baks:
	@echo "Restoring .bak files"
	@find . -name "*.rs.bak" | while read f; do mv -v "$$f" "$${f%.bak}"; done
