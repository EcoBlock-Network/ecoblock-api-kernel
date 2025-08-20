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

.PHONY: dev stop-dev

stop-dev:
	@echo "Stopping dev servers / freeing ports 3000, 5173, 5174 if occupied"
	@for p in 3000 5173 5174; do \
		pid=$$(lsof -tiTCP:$$p -sTCP:LISTEN || true); \
		if [ -n "$$pid" ]; then \
			echo "killing pid(s) on port $$p: $$pid"; \
			kill -9 $$pid || true; \
		fi; \
	done

dev: stop-dev
	@echo "Starting backend and web-admin dev server (logs -> /tmp)"
	@echo "Starting backend..."
	@nohup env RUST_LOG=info START_WEB_ADMIN=true VITE_DEV_TOKEN="${VITE_DEV_TOKEN:-}" cargo run -p $(PKG) > /tmp/ecoblock_backend.log 2>&1 & echo $$! > /tmp/ecoblock_backend.pid
	@sleep 2
	@echo "--- backend log tail ---"
	@tail -n 40 /tmp/ecoblock_backend.log || true
	@echo "Starting web-admin (vite)..."
	@cd web-admin && nohup env VITE_API_BASE=http://localhost:3000 npm run dev > /tmp/vite.log 2>&1 & echo $$! > /tmp/vite.pid
	@sleep 2
	@echo "--- vite log tail ---"
	@tail -n 80 /tmp/vite.log || true
	@echo "--- health check ---"
	@curl -sS -o /dev/null -w "%{http_code} %{time_total}s\n" http://localhost:3000/health || true
	@echo "--- GET /tangle/blocks with Origin http://localhost:5174 ---"
	@curl -i -H "Origin: http://localhost:5174" http://localhost:3000/tangle/blocks || true
