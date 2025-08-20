use reqwest::StatusCode;
use std::env;
use tokio::net::TcpListener;
use std::process::Command;

use ecoblock_api_kernel::db;
use ecoblock_api_kernel::kernel::build_app;
use ecoblock_api_kernel::plugins::tangle::plugin::TanglePlugin;
use base64::engine::general_purpose::STANDARD as BASE64_ENGINE;
use base64::Engine as _;

use ed25519_dalek::{Keypair, SecretKey, PublicKey, Signer};

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
    let tangle_plugin = TanglePlugin::new(pool.clone());
    let plugins: Vec<Box<dyn ecoblock_api_kernel::kernel::Plugin>> = vec![Box::new(ecoblock_api_kernel::plugins::health::HealthPlugin), Box::new(tangle_plugin)];
    let app = build_app(&plugins, None).await;

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let server_handle = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("server error");
    });

    let base = format!("http://{}", addr);
    Ok((base, server_handle, guard))
}

#[tokio::test]
async fn tangle_signature_valid_and_invalid() -> anyhow::Result<()> {
    let test_db = env::var("TEST_DATABASE_URL").unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/ecoblock_test".to_string());
    let (base, server_handle, _guard) = setup_http_and_spawn(&test_db).await?;
    let client = reqwest::Client::new();

    // prepare ed25519 keypair and data (deterministic to avoid rand_core version conflicts)
    let secret_bytes: [u8; 32] = [42u8; 32];
    let secret = SecretKey::from_bytes(&secret_bytes)?;
    let public = PublicKey::from(&secret);
    let keypair = Keypair { secret, public };
    let data = serde_json::json!({"hello":"signed world"});
    let msg = serde_json::to_vec(&data)?;
    let sig = keypair.sign(&msg).to_bytes();
    let pk_bytes = keypair.public.to_bytes();

    let payload_valid = serde_json::json!({
        "parents": ["p1"],
        "data": data,
        "signature": BASE64_ENGINE.encode(&sig),
        "public_key": BASE64_ENGINE.encode(&pk_bytes)
    });

    let create = client.post(&format!("{}/tangle", base)).json(&payload_valid).send().await?;
    assert_eq!(create.status(), StatusCode::OK, "valid signature should be accepted");

    // tamper signature (flip a byte)
    let mut bad_sig = sig;
    bad_sig[0] ^= 0xff;
    let payload_invalid = serde_json::json!({
        "parents": ["p1"],
        "data": serde_json::json!({"hello":"signed world"}),
        "signature": BASE64_ENGINE.encode(&bad_sig),
        "public_key": BASE64_ENGINE.encode(&pk_bytes)
    });

    let create_bad = client.post(&format!("{}/tangle", base)).json(&payload_invalid).send().await?;
    assert_eq!(create_bad.status(), StatusCode::BAD_REQUEST, "tampered signature should be rejected");

    server_handle.abort();
    let _ = server_handle.await;
    Ok(())
}
