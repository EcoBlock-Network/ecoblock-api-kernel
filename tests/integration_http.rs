use reqwest::StatusCode;
use serde_json::Value;
use std::env;
use tokio::net::TcpListener;

use ecoblock_api_kernel::db;
use ecoblock_api_kernel::kernel::build_app;

#[tokio::test]
async fn integration_crud_flow() -> anyhow::Result<()> {
    let test_db = env::var("TEST_DATABASE_URL").unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/ecoblock_test".to_string());

    // recreate DB
    let maintenance = test_db.clone();
    let mut maintenance_url = maintenance.clone();
    if let Some(idx) = maintenance_url.rfind('/') {
        maintenance_url.replace_range(idx + 1.., "postgres");
    }
    let db_name = test_db.rsplit('/').next().unwrap().split('?').next().unwrap();

    // drop/create
    let _ = std::process::Command::new("psql").arg(&maintenance_url).arg("-c").arg(format!("DROP DATABASE IF EXISTS \"{}\"", db_name)).status();
    let _ = std::process::Command::new("psql").arg(&maintenance_url).arg("-c").arg(format!("CREATE DATABASE \"{}\"", db_name)).status();
    let _ = std::process::Command::new("psql").arg(&test_db).arg("-c").arg("CREATE EXTENSION IF NOT EXISTS pgcrypto;").status();

    // init DB and run migrations in-process
    let pool = db::init_db(&test_db).await?;

    // build app with plugins
    let users_plugin = ecoblock_api_kernel::plugins::users::UsersPlugin::new(pool.clone());
    let plugins: Vec<Box<dyn ecoblock_api_kernel::kernel::Plugin>> = vec![Box::new(ecoblock_api_kernel::plugins::health::HealthPlugin), Box::new(users_plugin)];
    let app = build_app(&plugins).await;

    // bind to ephemeral port and spawn server
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let server_handle = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("server error");
    });

    let base = format!("http://{}", addr);
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
    let upd = client.put(&format!("{}/users/{}", base, id)).json(&serde_json::json!({"username":"ituser2","email":"it2@example.com"})).send().await?;
    assert_eq!(upd.status(), StatusCode::OK);

    // delete
    let del = client.delete(&format!("{}/users/{}", base, id)).send().await?;
    assert_eq!(del.status(), StatusCode::NO_CONTENT);

    // stop server
    server_handle.abort();
    let _ = server_handle.await;
    Ok(())
}
