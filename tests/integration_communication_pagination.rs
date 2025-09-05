use reqwest::StatusCode;
use serde_json::Value;
use std::env;
use std::process::Command;
use tokio::net::TcpListener;

use ecoblock_api_kernel::db;
use ecoblock_api_kernel::kernel::build_app;
use ecoblock_api_kernel::plugins::communication::blog::plugin::BlogPlugin;

struct TestDbGuard {
    maintenance_url: String,
    unique_db: String,
}

impl TestDbGuard {
    fn new(maintenance_url: String, unique_db: String) -> Self {
        Self {
            maintenance_url,
            unique_db,
        }
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

async fn setup_with_plugin(
    test_db: &str,
) -> anyhow::Result<(String, tokio::task::JoinHandle<()>, TestDbGuard)> {
    let maintenance = test_db.to_string();
    let mut maintenance_url = maintenance.clone();
    if let Some(idx) = maintenance_url.rfind('/') {
        maintenance_url.replace_range(idx + 1.., "postgres");
    }
    let base_db_name = test_db
        .rsplit('/')
        .next()
        .unwrap()
        .split('?')
        .next()
        .unwrap();
    let unique_db = format!(
        "{}_{}",
        base_db_name,
        uuid::Uuid::new_v4().to_string().replace('-', "")
    );
    let mut unique_db_url = test_db.to_string();
    if let Some(idx) = unique_db_url.rfind('/') {
        unique_db_url.replace_range(idx + 1.., &unique_db);
    }

    let _ = Command::new("psql")
        .arg(&maintenance_url)
        .arg("-c")
        .arg(format!("DROP DATABASE IF EXISTS \"{}\"", unique_db))
        .status();
    let _ = Command::new("psql")
        .arg(&maintenance_url)
        .arg("-c")
        .arg(format!("CREATE DATABASE \"{}\"", unique_db))
        .status();
    let _ = Command::new("psql")
        .arg(&unique_db_url)
        .arg("-c")
        .arg("CREATE EXTENSION IF NOT EXISTS pgcrypto;")
        .status();

    let guard = TestDbGuard::new(maintenance_url.clone(), unique_db.clone());

    let pool = db::init_db(&unique_db_url).await?;
    let blog_plugin = BlogPlugin::new(pool.clone());
    let plugins: Vec<Box<dyn ecoblock_api_kernel::kernel::Plugin>> = vec![
        Box::new(ecoblock_api_kernel::plugins::health::HealthPlugin),
        Box::new(blog_plugin),
    ];
    let app = build_app(&plugins, None, None, None).await;

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let server_handle = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("server error");
    });

    let base = format!("http://{}", addr);
    Ok((base, server_handle, guard))
}

#[tokio::test]
async fn pagination_blog_pages_and_limits() -> anyhow::Result<()> {
    let test_db = env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@localhost:5432/ecoblock_test".to_string()
    });
    let (base, server_handle, _guard) = setup_with_plugin(&test_db).await?;
    let client = reqwest::Client::new();

    // create 25 blogs
    for i in 0..25 {
        let _ = client.post(&format!("{}/communication/blog", base))
            .json(&serde_json::json!({"title":format!("t{}", i),"slug":format!("s{}", i),"body":"b","author":"a"}))
            .send()
            .await?;
    }

    // default per_page 20 -> page 1 should have 20, has_more true, total 25, total_pages 2
    let res = client
        .get(&format!("{}/communication/blog", base))
        .send()
        .await?;
    assert_eq!(res.status(), StatusCode::OK);
    let body: Value = res.json().await?;
    assert_eq!(body.get("page").and_then(|v| v.as_i64()), Some(1));
    assert_eq!(body.get("per_page").and_then(|v| v.as_i64()), Some(20));
    assert_eq!(body.get("total").and_then(|v| v.as_i64()), Some(25));
    assert_eq!(body.get("total_pages").and_then(|v| v.as_i64()), Some(2));
    assert_eq!(body.get("has_more").and_then(|v| v.as_bool()), Some(true));
    let items = body.get("items").and_then(|v| v.as_array()).unwrap();
    assert_eq!(items.len(), 20);

    // page 2 should have 5 items and has_more false
    let res2 = client
        .get(&format!("{}/communication/blog?page=2", base))
        .send()
        .await?;
    assert_eq!(res2.status(), StatusCode::OK);
    let body2: Value = res2.json().await?;
    assert_eq!(body2.get("page").and_then(|v| v.as_i64()), Some(2));
    assert_eq!(body2.get("has_more").and_then(|v| v.as_bool()), Some(false));
    let items2 = body2.get("items").and_then(|v| v.as_array()).unwrap();
    assert_eq!(items2.len(), 5);

    // per_page override to 10 -> page=1 returns 10, total_pages 3
    let res3 = client
        .get(&format!("{}/communication/blog?per_page=10", base))
        .send()
        .await?;
    assert_eq!(res3.status(), StatusCode::OK);
    let body3: Value = res3.json().await?;
    assert_eq!(body3.get("per_page").and_then(|v| v.as_i64()), Some(10));
    assert_eq!(body3.get("total_pages").and_then(|v| v.as_i64()), Some(3));

    server_handle.abort();
    let _ = server_handle.await;
    Ok(())
}
