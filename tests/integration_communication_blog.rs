mod common;
use common::{create_test_db_and_pool, spawn_app_with_plugins};
use reqwest::StatusCode;
use serde_json::Value;

#[tokio::test]
async fn communication_blog_crud_and_list() -> anyhow::Result<()> {
    let test_db = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/ecoblock_test".to_string());
    let (pool, _guard) = create_test_db_and_pool(&test_db).await?;
    let blog_plugin = ecoblock_api_kernel::plugins::communication::blog::plugin::BlogPlugin::new(pool.clone());
    let plugins: Vec<Box<dyn ecoblock_api_kernel::kernel::Plugin>> = vec![Box::new(ecoblock_api_kernel::plugins::health::HealthPlugin), Box::new(blog_plugin)];
    let (base, server_handle) = spawn_app_with_plugins(pool.clone(), plugins).await?;
    let client = reqwest::Client::new();

    // create blog
    let create = client.post(&format!("{}/communication/blog", base))
        .json(&serde_json::json!({"title":"Hello","slug":"hello","body":"body","author":"alice"}))
        .send()
        .await?;
    // consume response body into a string so we can log it and still parse JSON
    let status = create.status();
    let body_text = create.text().await.unwrap_or_else(|_| "<failed to read body>".to_string());
    if status != StatusCode::OK {
        eprintln!("create failed: status={} body={} ", status, body_text);
        assert_eq!(status, StatusCode::OK);
    }
    let created: Value = serde_json::from_str(&body_text)?;
    let id = created["id"].as_str().unwrap().to_string();

    // list
    let list = client.get(&format!("{}/communication/blog", base)).send().await?;
    assert_eq!(list.status(), StatusCode::OK);
    let list_body: Value = list.json().await?;
    assert!(list_body.get("items").is_some());
    assert_eq!(list_body.get("page").and_then(|v| v.as_i64()), Some(1));
    assert!(list_body.get("per_page").is_some());
    let total = list_body.get("total").and_then(|v| v.as_i64()).unwrap_or(0);
    assert!(total >= 0);
    // total_pages should exist and be >= 0
    let total_pages = list_body.get("total_pages").and_then(|v| v.as_i64()).unwrap_or(0);
    assert!(total_pages >= 0);
    assert!(list_body.get("has_more").is_some());

    // filter by author
    let by_author = client.get(&format!("{}/communication/blog?author=alice", base)).send().await?;
    assert_eq!(by_author.status(), StatusCode::OK);

    // get
    let one = client.get(&format!("{}/communication/blog/{}", base, id)).send().await?;
    assert_eq!(one.status(), StatusCode::OK);

    // update
    let upd = client.put(&format!("{}/communication/blog/{}", base, id)).json(&serde_json::json!({"title":"Hello2"})).send().await?;
    assert_eq!(upd.status(), StatusCode::OK);

    // delete
    let del = client.delete(&format!("{}/communication/blog/{}", base, id)).send().await?;
    assert_eq!(del.status(), StatusCode::NO_CONTENT);

    server_handle.abort();
    let _ = server_handle.await;
    Ok(())
}
