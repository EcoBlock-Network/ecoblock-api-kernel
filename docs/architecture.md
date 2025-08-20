# Architecture (résumé)

- Micro-kernel plugin architecture.
- Each plugin under `src/plugins/*` exposes `models`, `handlers`, `repo`, `plugin`.
- Database migrations in `migrations/` executed at test/run time.
- Observability via Prometheus metrics endpoint.

## Diagram rapide

- kernel -> mounts plugins -> axum router
- plugins -> handlers -> repo -> postgres (sqlx)

