use reqwest::StatusCode;
use serde_json::Value;
use std::env;
use std::process::{Child, Command};
use std::io::BufRead;

// reuse spawn_server logic from integration_http.rs
// now returns the Child and the JWT secret used so tests are self-contained
async fn spawn_server(database_url: &str) -> (Child, String) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("failed to bind ephemeral port");
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    // generate a per-test secret
    let secret = uuid::Uuid::new_v4().to_string();

    let mut cmd = Command::new("cargo");
    cmd.arg("run")
        .env("DATABASE_URL", database_url)
        .env("PORT", port.to_string())
        .env("JWT_SECRET", &secret)
        .stdout(std::process::Stdio::piped());
    let child = cmd.spawn().expect("failed to spawn server");
    (child, secret)
}

#[tokio::test]
async fn integration_auth_flow() -> anyhow::Result<()> {
    let test_db = env::var("TEST_DATABASE_URL").unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/ecoblock_test".to_string());

    // recreate DB
    let maintenance = test_db.clone();
    let mut maintenance_url = maintenance.clone();
    if let Some(idx) = maintenance_url.rfind('/') {
        maintenance_url.replace_range(idx + 1.., "postgres");
    }
    let db_name = test_db.rsplit('/').next().unwrap().split('?').next().unwrap();

    let _ = Command::new("psql").arg(&maintenance_url).arg("-c").arg(format!("DROP DATABASE IF EXISTS \"{}\"", db_name)).status();
    let _ = Command::new("psql").arg(&maintenance_url).arg("-c").arg(format!("CREATE DATABASE \"{}\"", db_name)).status();
    let _ = Command::new("psql").arg(&test_db).arg("-c").arg("CREATE EXTENSION IF NOT EXISTS pgcrypto;").status();

    // spawn server and read base URL. We receive back the secret used by the server
    let (mut child, jwt_secret) = spawn_server(&test_db).await;
    let stdout = child.stdout.take().expect("child had no stdout");
    let mut reader = std::io::BufReader::new(stdout);
    let mut line = String::new();
    let mut base = String::new();
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
    if base.is_empty() { base = "http://127.0.0.1:3000".to_string(); }

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

    let _ = child.kill();
    Ok(())
}
