use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_ENGINE;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize)]
pub struct TangleBlockCreate {
    pub id: Option<uuid::Uuid>,
    pub parents: Vec<String>,
    pub data: serde_json::Value,
    pub signature: String, // base64 encoded in payload
    pub public_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TangleBlockUpdate {
    pub parents: Option<Vec<String>>,
    pub data: Option<serde_json::Value>,
    pub signature: Option<String>,
    pub public_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TangleBlockDto {
    pub id: uuid::Uuid,
    pub parents: Vec<String>,
    pub data: serde_json::Value,
    // API-facing signature is base64 encoded
    pub signature: String,
    pub public_key: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// Internal DB representation (signature as bytes)
#[derive(Debug, sqlx::FromRow)]
pub struct TangleBlockRow {
    pub id: uuid::Uuid,
    pub parents: Vec<String>,
    pub data: serde_json::Value,
    pub signature: Vec<u8>,
    pub public_key: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<TangleBlockRow> for TangleBlockDto {
    fn from(r: TangleBlockRow) -> Self {
        TangleBlockDto {
            id: r.id,
            parents: r.parents,
            data: r.data,
            signature: BASE64_ENGINE.encode(&r.signature),
            public_key: r.public_key,
            created_at: r.created_at,
        }
    }
}
