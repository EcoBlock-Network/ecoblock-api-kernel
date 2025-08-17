# ecoblock-api-kernel

Quick start
1. Ensure Postgres is running and create a database for the app. By default this project uses:

```
postgres://postgres:postgres@localhost:5432/ecoblock
```

2. Apply sqlx migrations (from repo root):

```
export DATABASE_URL="postgres://postgres:postgres@localhost:5432/ecoblock"
sqlx migrate run
```

3. If your application connects with a non-superuser (for example `ecoblock`), grant it the required privileges. A migration `migrations/0003_grant_privs.sql` was added to do this automatically, or you can run the helper script:

```
# using the default role ecoblock
./scripts/db-setup.sh

# or with a custom role name
APP_DB_ROLE=myrole ./scripts/db-setup.sh
```

4. Start the server (generate a JWT secret for dev):

```
export JWT_SECRET=$(openssl rand -hex 32)
JWT_SECRET=$JWT_SECRET cargo run
```

5. In another shell, run the smoke-test script:

```
./scripts/test_endpoints.sh
```
