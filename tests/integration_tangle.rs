mod common;
use common::{create_test_db_and_pool, spawn_app_with_plugins};
use reqwest::StatusCode;
use serde_json::Value;
use base64::engine::general_purpose::STANDARD as BASE64_ENGINE;
use base64::Engine;

#[tokio::test]
async fn tangle_crud() -> anyhow::Result<()> {
    let test_db = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/ecoblock_test".to_string());
    let (pool, _guard) = create_test_db_and_pool(&test_db).await?;
    let tangle_plugin = ecoblock_api_kernel::plugins::tangle::plugin::TanglePlugin::new(pool.clone());
    let plugins: Vec<Box<dyn ecoblock_api_kernel::kernel::Plugin>> = vec![
        Box::new(ecoblock_api_kernel::plugins::health::HealthPlugin),
        Box::new(tangle_plugin),
    ];
    let (base, server_handle) = spawn_app_with_plugins(pool.clone(), plugins).await?;
    let client = reqwest::Client::new();

    // create (omit id, let DB generate UUID)
    let payload = serde_json::json!({
        "parents": ["p1","p2"],
        "data": {"hello":"world"},
    "signature": BASE64_ENGINE.encode(b"sigbytes"),
        "public_key": "pk"
    });

    let create = client.post(&format!("{}/tangle/blocks", base)).json(&payload).send().await?;
    assert_eq!(create.status(), StatusCode::OK);
    let created: Value = create.json().await?;
    let block_id_str = created["id"].as_str().unwrap().to_string();
    let block_uuid: uuid::Uuid = block_id_str.parse()?;

    // get
    let one = client.get(&format!("{}/tangle/{}", base, block_uuid)).send().await?;
    assert_eq!(one.status(), StatusCode::OK);

    // list
    let list = client.get(&format!("{}/tangle/blocks", base)).send().await?;
    assert_eq!(list.status(), StatusCode::OK);

    // update
    let upd = client
        .put(&format!("{}/tangle/{}", base, block_uuid))
        .json(&serde_json::json!({"data":{"foo":"bar"}}))
        .send()
        .await?;
    assert_eq!(upd.status(), StatusCode::OK);

    // delete
    let del = client.delete(&format!("{}/tangle/{}", base, block_uuid)).send().await?;
    assert_eq!(del.status(), StatusCode::NO_CONTENT);

    server_handle.abort();
    let _ = server_handle.await;
    Ok(())
}
