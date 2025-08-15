use reqwest::StatusCode;
use serde_json::Value;
use std::env;
use tokio::net::TcpListener;
use std::process::Command;

use ecoblock_api_kernel::db;
use ecoblock_api_kernel::kernel::build_app;
use std::sync::Once;

static JWT_INIT: Once = Once::new();
const JWT_SECRET_CONST: &str = "ecoblock-test-secret";

struct TestDbGuard {
    maintenance_url: String,
    unique_db: String,
}

impl TestDbGuard {
    fn new(maintenance_url: String, unique_db: String) -> Self {
        Self { maintenance_url, unique_db }
    }
}

impl Drop for TestDbGuard {
    fn drop(&mut self) {
        // Best-effort drop of the test database; ignore errors
        let _ = Command::new("psql")
            .arg(&self.maintenance_url)
            .arg("-c")
            .arg(format!(
                "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{}' AND pid <> pg_backend_pid();",
                self.unique_db
            ))
            .status();
        let _ = Command::new("psql")
            .arg(&self.maintenance_url)
            .arg("-c")
            .arg(format!("DROP DATABASE IF EXISTS \"{}\"", self.unique_db))
            .status();
    }
}

async fn setup_and_spawn(test_db: &str) -> anyhow::Result<(String, tokio::task::JoinHandle<()>, String, TestDbGuard)> {
    // compute maintenance connection and base db name
    let maintenance = test_db.to_string();
    let mut maintenance_url = maintenance.clone();
    if let Some(idx) = maintenance_url.rfind('/') {
        maintenance_url.replace_range(idx + 1.., "postgres");
    }
    let base_db_name = test_db.rsplit('/').next().unwrap().split('?').next().unwrap();

    // create a unique DB name for this test
    let unique_db = format!("{}_{}", base_db_name, uuid::Uuid::new_v4().to_string().replace('-', ""));
    let mut unique_db_url = test_db.to_string();
    if let Some(idx) = unique_db_url.rfind('/') {
        unique_db_url.replace_range(idx + 1.., &unique_db);
    }

    // drop/create the unique DB and create extension
    let _ = Command::new("psql").arg(&maintenance_url).arg("-c").arg(format!("DROP DATABASE IF EXISTS \"{}\"", unique_db)).status();
    let _ = Command::new("psql").arg(&maintenance_url).arg("-c").arg(format!("CREATE DATABASE \"{}\"", unique_db)).status();
    let _ = Command::new("psql").arg(&unique_db_url).arg("-c").arg("CREATE EXTENSION IF NOT EXISTS pgcrypto;").status();

    // create guard that will DROP the unique DB when it goes out of scope
    let guard = TestDbGuard::new(maintenance_url.clone(), unique_db.clone());

    // ensure a stable JWT secret is set once for the test process to avoid races
    JWT_INIT.call_once(|| {
        unsafe { std::env::set_var("JWT_SECRET", JWT_SECRET_CONST); }
    });
    let jwt_secret = JWT_SECRET_CONST.to_string();

    // init DB and run migrations in-process against unique DB
    let pool = db::init_db(&unique_db_url).await?;

    // build app with plugins
    let users_plugin = ecoblock_api_kernel::plugins::users::UsersPlugin::new(pool.clone());
    let auth_plugin = ecoblock_api_kernel::plugins::auth::AuthPlugin::new(pool.clone());
    let plugins: Vec<Box<dyn ecoblock_api_kernel::kernel::Plugin>> = vec![Box::new(ecoblock_api_kernel::plugins::health::HealthPlugin), Box::new(users_plugin), Box::new(auth_plugin)];
    let app = build_app(&plugins).await;

    // bind to ephemeral port and spawn server using axum
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let server_handle = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("server error");
    });

    let base = format!("http://{}", addr);
    Ok((base, server_handle, jwt_secret, guard))
}

#[tokio::test]
async fn integration_auth_flow() -> anyhow::Result<()> {
    let test_db = env::var("TEST_DATABASE_URL").unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/ecoblock_test".to_string());
    let (base, server_handle, jwt_secret, _guard) = setup_and_spawn(&test_db).await?;
    let client = reqwest::Client::new();

    // create user
    let create = client.post(&format!("{}/users", base))
        .json(&serde_json::json!({"username":"ituser_auth","email":"it_auth@example.com","password":"password123"}))
        .send()
        .await?;
    assert_eq!(create.status(), StatusCode::OK);
    let created: Value = create.json().await?;
    let created_id = created["id"].as_str().unwrap().to_string();

    // login
    let login = client.post(&format!("{}/auth/login", base))
        .json(&serde_json::json!({"username":"ituser_auth","password":"password123"}))
        .send()
        .await?;
    assert_eq!(login.status(), StatusCode::OK);
    let token_body: Value = login.json().await?;
    let token = token_body["token"].as_str().unwrap();

    // decode token and assert sub equals created id
    #[derive(serde::Deserialize)]
    struct Claims { sub: String }
    let token_data = jsonwebtoken::decode::<Claims>(token, &jsonwebtoken::DecodingKey::from_secret(jwt_secret.as_bytes()), &jsonwebtoken::Validation::default())?;
    assert_eq!(token_data.claims.sub, created_id);

    // whoami
    let who = client.get(&format!("{}/auth/whoami", base))
        .bearer_auth(token)
        .send()
        .await?;
    assert_eq!(who.status(), StatusCode::OK);
    let who_body: Value = who.json().await?;
    assert_eq!(who_body["username"].as_str().unwrap(), "ituser_auth");
    assert_eq!(who_body["email"].as_str().unwrap(), "it_auth@example.com");

    // stop the server task; the TestDbGuard will drop the database on scope exit
    server_handle.abort();
    let _ = server_handle.await;
    Ok(())
}

#[tokio::test]
async fn invalid_credentials_returns_401_and_code() -> anyhow::Result<()> {
    let test_db = env::var("TEST_DATABASE_URL").unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/ecoblock_test".to_string());
    let (base, server_handle, _jwt_secret, _guard) = setup_and_spawn(&test_db).await?;
    let client = reqwest::Client::new();

    // attempt login with non-existing user
    let login = client.post(&format!("{}/auth/login", base))
        .json(&serde_json::json!({"username":"no_such","password":"bad"}))
        .send()
        .await?;
    assert_eq!(login.status(), StatusCode::UNAUTHORIZED);
    let body: Value = login.json().await?;
    assert_eq!(body["code"].as_str().unwrap(), "invalid_credentials");

    server_handle.abort();
    let _ = server_handle.await;
    Ok(())
}

#[tokio::test]
async fn malformed_token_returns_401_invalid_token() -> anyhow::Result<()> {
    let test_db = env::var("TEST_DATABASE_URL").unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/ecoblock_test".to_string());
    let (base, server_handle, _jwt_secret, _guard) = setup_and_spawn(&test_db).await?;
    let client = reqwest::Client::new();

    // call whoami with a malformed token
    let who = client.get(&format!("{}/auth/whoami", base))
        .bearer_auth("not-a-jwt")
        .send()
        .await?;
    assert_eq!(who.status(), StatusCode::UNAUTHORIZED);
    let body: Value = who.json().await?;
    assert_eq!(body["code"].as_str().unwrap(), "invalid_token");

    server_handle.abort();
    let _ = server_handle.await;
    Ok(())
}

#[tokio::test]
async fn expired_token_returns_401() -> anyhow::Result<()> {
    let test_db = env::var("TEST_DATABASE_URL").unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/ecoblock_test".to_string());
    let (base, server_handle, jwt_secret, _guard) = setup_and_spawn(&test_db).await?;
    let client = reqwest::Client::new();

    // create a user
    let create = client.post(&format!("{}/users", base))
        .json(&serde_json::json!({"username":"ituser_exp","email":"it_exp@example.com","password":"password123"}))
        .send()
        .await?;
    assert_eq!(create.status(), StatusCode::OK);
    let created: Value = create.json().await?;
    let created_id = created["id"].as_str().unwrap().to_string();

    // create an expired token manually (exp in the past)
    let exp = (chrono::Utc::now() - chrono::Duration::hours(1)).timestamp() as usize;
    let claims = serde_json::json!({ "sub": created_id.clone(), "exp": exp });
    let token = jsonwebtoken::encode(&jsonwebtoken::Header::default(), &claims, &jsonwebtoken::EncodingKey::from_secret(jwt_secret.as_bytes()))?;

    // whoami with expired token
    let who = client.get(&format!("{}/auth/whoami", base))
        .bearer_auth(&token)
        .send()
        .await?;
    assert_eq!(who.status(), StatusCode::UNAUTHORIZED);
    let body: Value = who.json().await?;
    assert_eq!(body["code"].as_str().unwrap(), "invalid_token");

    server_handle.abort();
    let _ = server_handle.await;
    Ok(())
}
