mod common;
use common::{create_test_db_and_pool, spawn_app_with_plugins};
use reqwest::StatusCode;
use serde_json::Value;

#[tokio::test]
async fn integration_crud_flow() -> anyhow::Result<()> {
    let test_db = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@localhost:5432/ecoblock_test".to_string()
    });
    let (pool, _guard) = create_test_db_and_pool(&test_db).await?;
    let users_plugin = ecoblock_api_kernel::plugins::users::UsersPlugin::new(pool.clone());
    let plugins: Vec<Box<dyn ecoblock_api_kernel::kernel::Plugin>> = vec![
        Box::new(ecoblock_api_kernel::plugins::health::HealthPlugin),
        Box::new(users_plugin),
    ];
    let (base, server_handle) = spawn_app_with_plugins(pool.clone(), plugins).await?;
    let client = reqwest::Client::new();

    // create
    let create = client.post(&format!("{}/users", base))
        .json(&serde_json::json!({"username":"ituser","email":"it@example.com","password":"password123"}))
        .send()
        .await?;
    assert_eq!(create.status(), StatusCode::OK);
    let created: Value = create.json().await?;
    let id = created["id"].as_str().unwrap();

    // duplicate create -> expect 409 with code
    let dup = client.post(&format!("{}/users", base))
        .json(&serde_json::json!({"username":"ituser","email":"it@example.com","password":"password123"}))
        .send()
        .await?;
    assert_eq!(dup.status(), StatusCode::CONFLICT);
    let err: Value = dup.json().await?;
    assert!(err.get("code").is_some());

    // get
    let list = client.get(&format!("{}/users", base)).send().await?;
    assert_eq!(list.status(), StatusCode::OK);

    // get by id
    let one = client.get(&format!("{}/users/{}", base, id)).send().await?;
    assert_eq!(one.status(), StatusCode::OK);

    // update
    let upd = client
        .put(&format!("{}/users/{}", base, id))
        .json(&serde_json::json!({"username":"ituser2","email":"it2@example.com"}))
        .send()
        .await?;
    assert_eq!(upd.status(), StatusCode::OK);

    // delete
    let del = client
        .delete(&format!("{}/users/{}", base, id))
        .send()
        .await?;
    assert_eq!(del.status(), StatusCode::NO_CONTENT);

    server_handle.abort();
    let _ = server_handle.await;
    Ok(())
}
