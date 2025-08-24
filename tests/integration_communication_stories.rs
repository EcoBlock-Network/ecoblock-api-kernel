use reqwest::StatusCode;
use serde_json::Value;
use std::env;
use tokio::net::TcpListener;
use std::process::Command;

use ecoblock_api_kernel::db;
use ecoblock_api_kernel::kernel::build_app;
use ecoblock_api_kernel::plugins::communication::stories::plugin::StoriesPlugin;

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

async fn setup_http_and_spawn(test_db: &str) -> anyhow::Result<(String, tokio::task::JoinHandle<()>, TestDbGuard)> {
    let maintenance = test_db.to_string();
    let mut maintenance_url = maintenance.clone();
    if let Some(idx) = maintenance_url.rfind('/') {
        maintenance_url.replace_range(idx + 1.., "postgres");
    }
    let base_db_name = test_db.rsplit('/').next().unwrap().split('?').next().unwrap();
    let unique_db = format!("{}_{}", base_db_name, uuid::Uuid::new_v4().to_string().replace('-', ""));
    let mut unique_db_url = test_db.to_string();
    if let Some(idx) = unique_db_url.rfind('/') {
        unique_db_url.replace_range(idx + 1.., &unique_db);
    }

    let _ = Command::new("psql").arg(&maintenance_url).arg("-c").arg(format!("DROP DATABASE IF EXISTS \"{}\"", unique_db)).status();
    let _ = Command::new("psql").arg(&maintenance_url).arg("-c").arg(format!("CREATE DATABASE \"{}\"", unique_db)).status();
    let _ = Command::new("psql").arg(&unique_db_url).arg("-c").arg("CREATE EXTENSION IF NOT EXISTS pgcrypto;").status();

    let guard = TestDbGuard::new(maintenance_url.clone(), unique_db.clone());

    let pool = db::init_db(&unique_db_url).await?;
    let stories_plugin = StoriesPlugin::new(pool.clone());
    let plugins: Vec<Box<dyn ecoblock_api_kernel::kernel::Plugin>> = vec![Box::new(ecoblock_api_kernel::plugins::health::HealthPlugin), Box::new(stories_plugin)];
    let app = build_app(&plugins, None, None).await;

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let server_handle = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("server error");
    });

    let base = format!("http://{}", addr);
    Ok((base, server_handle, guard))
}

#[tokio::test]
async fn communication_stories_crud_and_list() -> anyhow::Result<()> {
    let test_db = env::var("TEST_DATABASE_URL").unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/ecoblock_test".to_string());
    let (base, server_handle, _guard) = setup_http_and_spawn(&test_db).await?;
    let client = reqwest::Client::new();

    // create story
    let create = client.post(&format!("{}/communication/stories", base))
        .json(&serde_json::json!({"media_url":"https://example.com/m.png","caption":"hi"}))
        .send()
        .await?;
    assert_eq!(create.status(), StatusCode::OK);
    let created: Value = create.json().await?;
    let id = created["id"].as_str().unwrap().to_string();

    // list
    let list = client.get(&format!("{}/communication/stories", base)).send().await?;
    assert_eq!(list.status(), StatusCode::OK);
    let list_body: Value = list.json().await?;
    assert!(list_body.get("items").is_some());
    assert_eq!(list_body.get("page").and_then(|v| v.as_i64()), Some(1));
    assert!(list_body.get("per_page").is_some());
    let total = list_body.get("total").and_then(|v| v.as_i64()).unwrap_or(0);
    assert!(total >= 0);
    let total_pages = list_body.get("total_pages").and_then(|v| v.as_i64()).unwrap_or(0);
    assert!(total_pages >= 0);
    assert!(list_body.get("has_more").is_some());

    // filter by created_by (admin used in handlers)
    let by_creator = client.get(&format!("{}/communication/stories?created_by=admin", base)).send().await?;
    assert_eq!(by_creator.status(), StatusCode::OK);

    // get
    let one = client.get(&format!("{}/communication/stories/{}", base, id)).send().await?;
    assert_eq!(one.status(), StatusCode::OK);

    // update
    let upd = client.put(&format!("{}/communication/stories/{}", base, id)).json(&serde_json::json!({"caption":"bye"})).send().await?;
    assert_eq!(upd.status(), StatusCode::OK);

    // delete
    let del = client.delete(&format!("{}/communication/stories/{}", base, id)).send().await?;
    assert_eq!(del.status(), StatusCode::NO_CONTENT);

    server_handle.abort();
    let _ = server_handle.await;
    Ok(())
}
