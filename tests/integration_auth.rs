mod common;
use common::setup_and_spawn;
use reqwest::StatusCode;
use serde_json::Value;

#[tokio::test]
async fn integration_auth_flow() -> anyhow::Result<()> {
    let test_db = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@localhost:5432/ecoblock_test".to_string()
    });
    let (base, server_handle, jwt_secret, _pool, _guard) = setup_and_spawn(&test_db).await?;
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
    let login = client
        .post(&format!("{}/auth/login", base))
        .json(&serde_json::json!({"username":"ituser_auth","password":"password123"}))
        .send()
        .await?;
    assert_eq!(login.status(), StatusCode::OK);
    let token_body: Value = login.json().await?;
    let token = token_body["token"].as_str().unwrap();

    // decode token and assert sub equals created id
    #[derive(serde::Deserialize)]
    struct Claims {
        sub: String,
    }
    let token_data = jsonwebtoken::decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(jwt_secret.as_bytes()),
        &jsonwebtoken::Validation::default(),
    )?;
    assert_eq!(token_data.claims.sub, created_id);

    // whoami
    let who = client
        .get(&format!("{}/auth/whoami", base))
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
    let test_db = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@localhost:5432/ecoblock_test".to_string()
    });
    let (base, server_handle, _jwt_secret, _pool, _guard) = setup_and_spawn(&test_db).await?;
    let client = reqwest::Client::new();

    // attempt login with non-existing user
    let login = client
        .post(&format!("{}/auth/login", base))
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
    let test_db = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@localhost:5432/ecoblock_test".to_string()
    });
    let (base, server_handle, _jwt_secret, _pool, _guard) = setup_and_spawn(&test_db).await?;
    let client = reqwest::Client::new();

    // call whoami with a malformed token
    let who = client
        .get(&format!("{}/auth/whoami", base))
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
    let test_db = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@localhost:5432/ecoblock_test".to_string()
    });
    let (base, server_handle, jwt_secret, _pool, _guard) = setup_and_spawn(&test_db).await?;
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
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(jwt_secret.as_bytes()),
    )?;

    // whoami with expired token
    let who = client
        .get(&format!("{}/auth/whoami", base))
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
