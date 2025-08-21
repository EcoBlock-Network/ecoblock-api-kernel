use axum::{extract::Path, Json, Extension};
use axum::http::StatusCode;
use crate::http_error::AppError;
use crate::plugins::tangle::models::{TangleBlockCreate, TangleBlockUpdate, TangleBlockDto};
use crate::plugins::tangle::repo as repo;
use crate::plugins::tangle::crypto::verify_ed25519_signature;
use sqlx::PgPool;
use base64::engine::general_purpose::STANDARD as BASE64_ENGINE;
use base64::Engine as _;

pub async fn create_block(Extension(pool): Extension<PgPool>, Json(payload): Json<TangleBlockCreate>) -> Result<Json<TangleBlockDto>, AppError> {
    // validate base64 signature and decode to bytes for storage
    let sig_bytes = BASE64_ENGINE.decode(&payload.signature).map_err(|e| AppError::new(axum::http::StatusCode::BAD_REQUEST, format!("invalidSignatureEncoding: {}", e)))?;
    // if public_key is base64 and decodes to 32 bytes we attempt Ed25519 verification
    if let Ok(pk_bytes) = BASE64_ENGINE.decode(&payload.public_key) {
        if pk_bytes.len() == 32 {
            // verify signature over canonical JSON bytes of `data`
            let msg = serde_json::to_vec(&payload.data).map_err(|e| AppError::new(axum::http::StatusCode::BAD_REQUEST, format!("invalidDataPayload: {}", e)))?;
            match verify_ed25519_signature(&pk_bytes, &msg, &sig_bytes) {
                Ok(true) => {
                    // ok
                }
                Ok(false) => {
                    return Err(AppError::new(axum::http::StatusCode::BAD_REQUEST, "signatureVerificationFailed".to_string()));
                }
                Err(_) => {
                    return Err(AppError::new(axum::http::StatusCode::BAD_REQUEST, "signatureVerificationError".to_string()));
                }
            }
        }
    }
    // allow server-generated UUID if not provided
    let row = repo::insert_block(&pool, payload.id, &payload.parents, &payload.data, &sig_bytes, &payload.public_key).await?;
    Ok(Json(row.into()))
}

pub async fn get_block(Extension(pool): Extension<PgPool>, Path(id): Path<uuid::Uuid>) -> Result<Json<TangleBlockDto>, AppError> {
    let row = repo::get_block(&pool, id).await?;
    Ok(Json(row.into()))
}

#[derive(Debug, serde::Deserialize)]
pub struct ListQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

pub async fn list_blocks(Extension(pool): Extension<PgPool>, axum::extract::Query(q): axum::extract::Query<ListQuery>) -> Result<Json<crate::plugins::communication::shared::ListResponse<TangleBlockDto>>, AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let per_page = q.per_page.unwrap_or(20).clamp(1,100);
    let offset = ((page - 1) * per_page) as i64;

    let (rows, total) = repo::list_blocks(&pool, per_page as i64, offset).await?;
    let items: Vec<TangleBlockDto> = rows.into_iter().map(|r| r.into()).collect();

    let fetched = items.len() as i64;
    let has_more = offset + fetched < total;
    let total_pages = if total == 0 { 0 } else { ((total as f64) / (per_page as f64)).ceil() as i64 };
    let resp = crate::plugins::communication::shared::ListResponse { items, page, per_page, total, total_pages, has_more };
    Ok(Json(resp))
}

pub async fn update_block(Extension(pool): Extension<PgPool>, Path(id): Path<uuid::Uuid>, Json(payload): Json<TangleBlockUpdate>) -> Result<Json<TangleBlockDto>, AppError> {
    // decode signature if provided and store bytes
    let sig_bytes_opt: Option<Vec<u8>> = match &payload.signature {
        Some(s) => Some(BASE64_ENGINE.decode(s).map_err(|e| AppError::new(axum::http::StatusCode::BAD_REQUEST, format!("invalidSignatureEncoding: {}", e)))?),
        None => None,
    };

    let row = repo::update_block(&pool, id, payload.parents, payload.data, sig_bytes_opt, payload.public_key).await?;
    Ok(Json(row.into()))
}

pub async fn delete_block(Extension(pool): Extension<PgPool>, Path(id): Path<uuid::Uuid>) -> Result<StatusCode, AppError> {
    repo::delete_block(&pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
