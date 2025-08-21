mod common;
use common::{create_test_db_and_pool, spawn_app_with_plugins};
use reqwest::StatusCode;
use base64::engine::general_purpose::STANDARD as BASE64_ENGINE;
use base64::Engine as _;
use ed25519_dalek::{Keypair, SecretKey, PublicKey, Signer};

#[tokio::test]
async fn tangle_signature_valid_and_invalid() -> anyhow::Result<()> {
    let test_db = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/ecoblock_test".to_string());
    let (pool, _guard) = create_test_db_and_pool(&test_db).await?;
    let tangle_plugin = ecoblock_api_kernel::plugins::tangle::plugin::TanglePlugin::new(pool.clone());
    let plugins: Vec<Box<dyn ecoblock_api_kernel::kernel::Plugin>> = vec![Box::new(ecoblock_api_kernel::plugins::health::HealthPlugin), Box::new(tangle_plugin)];
    let (base, server_handle) = spawn_app_with_plugins(pool.clone(), plugins).await?;
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
