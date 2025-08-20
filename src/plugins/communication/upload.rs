use axum::{Router, routing::post, extract::Multipart, Json};
use crate::http_error::AppError;
use axum::http::StatusCode;
use std::path::PathBuf;
use uuid::Uuid;

pub async fn upload_file(mut multipart: Multipart) -> Result<Json<serde_json::Value>, AppError> {
    // ensure upload dir exists
    let mut uploaded_urls: Vec<String> = Vec::new();
    let base_dir = std::path::Path::new("data/uploads");
    if !base_dir.exists() {
        std::fs::create_dir_all(base_dir).map_err(|e| AppError::from((StatusCode::INTERNAL_SERVER_ERROR, format!("failed to create upload dir: {}", e))))?;
    }

    while let Some(field) = multipart.next_field().await.map_err(|e| AppError::from((StatusCode::BAD_REQUEST, format!("multipart error: {}", e))))? {
        if let Some(filename_orig) = field.file_name() {
            let filename = filename_orig.to_string();
            let data = field.bytes().await.map_err(|e| AppError::from((StatusCode::BAD_REQUEST, format!("multipart read error: {}", e))))?;
            let ext = std::path::Path::new(&filename).extension().and_then(|s| s.to_str()).unwrap_or("bin");
            let fname = format!("{}-{}.{}", Uuid::new_v4(), chrono::Utc::now().timestamp(), ext);
            let mut path = PathBuf::from(base_dir);
            path.push(&fname);
            tokio::fs::write(&path, &data).await.map_err(|e| AppError::from((StatusCode::INTERNAL_SERVER_ERROR, format!("write error: {}", e))))?;
            let url = format!("/uploads/{}", fname);
            uploaded_urls.push(url);
        }
    }

    Ok(Json(serde_json::json!({ "uploaded": uploaded_urls })))
}

pub fn router() -> Router {
    Router::new().route("/upload", post(upload_file))
}
