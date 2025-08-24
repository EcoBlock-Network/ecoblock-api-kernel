mod common;
use common::setup_and_spawn;
use reqwest::StatusCode;
use serde_json::Value;
use uuid::Uuid;

#[tokio::test]
async fn integration_create_admin_flow() -> anyhow::Result<()> {
    let test_db = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@localhost:5432/ecoblock_test".to_string()
    });
    let (base, server_handle, _jwt_secret, pool, _guard) = setup_and_spawn(&test_db).await?;
    let client = reqwest::Client::new();
    let _ = ecoblock_api_kernel::plugins::users::repo::insert_user_with_admin(
        &pool,
        "itadmin",
        "itadmin@example.com",
        "password123",
        true,
    )
    .await
    .map_err(|e| anyhow::anyhow!("repo error: {:?}", e))?;
    let login = client
        .post(&format!("{}/auth/login", base))
        .json(&serde_json::json!({"username":"itadmin","password":"password123"}))
        .send()
        .await?;
    assert_eq!(login.status(), StatusCode::OK);
    let token_body: Value = login.json().await?;
    let token = token_body["token"].as_str().unwrap();
    let create = client.post(&format!("{}/users/admin", base)).bearer_auth(token).json(&serde_json::json!({"username":"newadmin","email":"newadmin@example.com","password":"password123"})).send().await?;
    assert_eq!(create.status(), StatusCode::OK);
    let created: Value = create.json().await?;
    let created_id = created["id"].as_str().unwrap().to_string();
    let created_uuid = Uuid::parse_str(&created_id)?;
    let is_admin = ecoblock_api_kernel::plugins::users::repo::is_admin(&pool, created_uuid)
        .await
        .map_err(|e| anyhow::anyhow!("repo error: {:?}", e))?;
    assert!(is_admin, "expected created user to be admin");
    server_handle.abort();
    let _ = server_handle.await;
    Ok(())
}

#[tokio::test]
async fn integration_create_admin_forbidden_for_non_admin() -> anyhow::Result<()> {
    let test_db = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@localhost:5432/ecoblock_test".to_string()
    });
    let (base, server_handle, _jwt_secret, pool, _guard) = setup_and_spawn(&test_db).await?;
    let client = reqwest::Client::new();
    let _ = ecoblock_api_kernel::plugins::users::repo::insert_user(
        &pool,
        "regular",
        "regular@example.com",
        "password123",
    )
    .await
    .map_err(|e| anyhow::anyhow!("repo error: {:?}", e))?;
    let login = client
        .post(&format!("{}/auth/login", base))
        .json(&serde_json::json!({"username":"regular","password":"password123"}))
        .send()
        .await?;
    assert_eq!(login.status(), StatusCode::OK);
    let token_body: Value = login.json().await?;
    let token = token_body["token"].as_str().unwrap();
    let create = client.post(&format!("{}/users/admin", base)).bearer_auth(token).json(&serde_json::json!({"username":"shouldfail","email":"shouldfail@example.com","password":"password123"})).send().await?;
    assert_eq!(create.status(), StatusCode::FORBIDDEN);
    let body: Value = create.json().await?;
    assert_eq!(
        body["error"].as_str().unwrap(),
        "onlyAdminCanCreateAdminAccounts"
    );
    server_handle.abort();
    let _ = server_handle.await;
    Ok(())
}

#[tokio::test]
async fn integration_create_admin_validation_errors() -> anyhow::Result<()> {
    let test_db = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@localhost:5432/ecoblock_test".to_string()
    });
    let (base, server_handle, _jwt_secret, pool, _guard) = setup_and_spawn(&test_db).await?;
    let client = reqwest::Client::new();
    let _ = ecoblock_api_kernel::plugins::users::repo::insert_user_with_admin(
        &pool,
        "itadmin2",
        "itadmin2@example.com",
        "password123",
        true,
    )
    .await
    .map_err(|e| anyhow::anyhow!("repo error: {:?}", e))?;
    let login = client
        .post(&format!("{}/auth/login", base))
        .json(&serde_json::json!({"username":"itadmin2","password":"password123"}))
        .send()
        .await?;
    assert_eq!(login.status(), StatusCode::OK);
    let token_body: Value = login.json().await?;
    let token = token_body["token"].as_str().unwrap();
    let create_bad_email = client
        .post(&format!("{}/users/admin", base))
        .bearer_auth(token)
        .json(&serde_json::json!({"username":"bad","email":"bad-email","password":"password123"}))
        .send()
        .await?;
    assert_eq!(create_bad_email.status(), StatusCode::BAD_REQUEST);
    let body_email: Value = create_bad_email.json().await?;
    assert_eq!(body_email["error"].as_str().unwrap(), "invalidEmail");
    let create_short_pwd = client
        .post(&format!("{}/users/admin", base))
        .bearer_auth(token)
        .json(&serde_json::json!({"username":"bad2","email":"bad2@example.com","password":"short"}))
        .send()
        .await?;
    assert_eq!(create_short_pwd.status(), StatusCode::BAD_REQUEST);
    let body_pwd: Value = create_short_pwd.json().await?;
    assert_eq!(body_pwd["error"].as_str().unwrap(), "passwordTooShort");
    server_handle.abort();
    let _ = server_handle.await;
    Ok(())
}

#[tokio::test]
async fn integration_create_user_duplicate_email_returns_409() -> anyhow::Result<()> {
    let test_db = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@localhost:5432/ecoblock_test".to_string()
    });
    let (base, server_handle, _jwt_secret, _pool, _guard) = setup_and_spawn(&test_db).await?;
    let client = reqwest::Client::new();
    // create first user
    let create = client.post(&format!("{}/users", base)).json(&serde_json::json!({"username":"dupe","email":"dupe@example.com","password":"password123"})).send().await?;
    assert_eq!(create.status(), StatusCode::OK);
    // create duplicate email
    let create2 = client.post(&format!("{}/users", base)).json(&serde_json::json!({"username":"dupe2","email":"dupe@example.com","password":"password123"})).send().await?;
    assert_eq!(create2.status(), StatusCode::CONFLICT);
    let body: serde_json::Value = create2.json().await?;
    assert_eq!(body["code"].as_str().unwrap_or(""), "duplicate_email");
    server_handle.abort();
    let _ = server_handle.await;
    Ok(())
}
