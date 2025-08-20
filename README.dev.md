Development notes

- Use the included `docker-compose.yml` to run Postgres locally for integration testing.
- Export `DATABASE_URL` to point at the test DB when running locally, or set `TEST_DATABASE_URL`.
- Run the full test suite with `cargo test --workspace`.
- To build the Docker image locally:
  - docker build -t ecoblock-api-kernel:local .
- CI: GitHub Actions runs tests, clippy, and cargo-audit.
