use reqwest::StatusCode;
use serde_json::Value;
use std::env;
use std::process::{Child, Command};
use std::io::BufRead;
// ...existing code...

// start app as background process for integration test
async fn spawn_server(database_url: &str) -> Child {
    // pick a random free port by asking the OS
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("failed to bind ephemeral port");
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    let mut cmd = Command::new("cargo");
    cmd.arg("run").env("DATABASE_URL", database_url).env("PORT", port.to_string()).stdout(std::process::Stdio::piped());
    cmd.spawn().expect("failed to spawn server")
}

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
    let _ = Command::new("psql").arg(&maintenance_url).arg("-c").arg(format!("DROP DATABASE IF EXISTS \"{}\"", db_name)).status();
    let _ = Command::new("psql").arg(&maintenance_url).arg("-c").arg(format!("CREATE DATABASE \"{}\"", db_name)).status();
    let _ = Command::new("psql").arg(&test_db).arg("-c").arg("CREATE EXTENSION IF NOT EXISTS pgcrypto;").status();

    // spawn server on an ephemeral port and capture stdout to read listening address
    let mut child = spawn_server(&test_db).await;
    // read stdout to find the listening address
    let stdout = child.stdout.take().expect("child had no stdout");
    let mut reader = std::io::BufReader::new(stdout);
    let mut line = String::new();
    let mut base = String::new();
    // read lines until we find "listening on"
    loop {
        line.clear();
        let n = reader.read_line(&mut line).unwrap_or(0);
        if n == 0 { break; }
        if line.contains("listening on") {
            if let Some(idx) = line.rfind(':') {
                let port = line[idx+1..].trim();
                base = format!("http://127.0.0.1:{}", port);
                break;
            }
        }
    }

    if base.is_empty() {
        // fallback
        base = "http://127.0.0.1:3000".to_string();
    }

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

    // teardown
    let _ = child.kill();
    Ok(())
}
