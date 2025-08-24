use crate::http_error::AppError;
use axum::http::StatusCode;
use axum::{Json, Router, extract::Multipart, routing::post};
use image::ImageOutputFormat;
use image::io::Reader as ImageReader;
use infer;
use std::path::PathBuf;
use uuid::Uuid;

pub async fn upload_file(mut multipart: Multipart) -> Result<Json<serde_json::Value>, AppError> {
    let mut uploaded_urls: Vec<String> = Vec::new();
    let base_dir = std::path::Path::new("data/uploads");
    if !base_dir.exists() {
        std::fs::create_dir_all(base_dir).map_err(|e| {
            AppError::from((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to create upload dir: {}", e),
            ))
        })?;
    }

    const MAX_SIZE: usize = 10 * 1024 * 1024;
    const MAX_DIM: u32 = 2000;
    let allowed_mimes = vec!["image/png", "image/jpeg", "image/gif", "image/webp"];

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::from((StatusCode::BAD_REQUEST, format!("multipart error: {}", e))))?
    {
        if let Some(filename_orig) = field.file_name() {
            let filename = filename_orig.to_string();
            let data = field.bytes().await.map_err(|e| {
                AppError::from((
                    StatusCode::BAD_REQUEST,
                    format!("multipart read error: {}", e),
                ))
            })?;
            if data.len() > MAX_SIZE {
                return Err(AppError::from((
                    StatusCode::BAD_REQUEST,
                    format!("file too large (max {} MB)", MAX_SIZE / 1024 / 1024),
                )));
            }
            let kind = infer::get(&data);
            let mime = kind
                .map(|k| k.mime_type())
                .unwrap_or("application/octet-stream");
            if !allowed_mimes.contains(&mime) {
                return Err(AppError::from((
                    StatusCode::BAD_REQUEST,
                    format!("unsupported file type: {}", mime),
                )));
            }
            let ext = match mime {
                "image/png" => "png",
                "image/jpeg" => "jpg",
                "image/gif" => "gif",
                "image/webp" => "webp",
                _ => {
                    std::path::Path::new(&filename)
                        .extension()
                        .and_then(|s| s.to_str())
                        .unwrap_or("bin")
                }
            };
            let mut final_bytes = data.to_vec();
            if mime.starts_with("image/") {
                if let Ok(reader) =
                    ImageReader::new(std::io::Cursor::new(&data)).with_guessed_format()
                {
                    if let Ok(img) = reader.decode() {
                        let w = img.width();
                        let h = img.height();
                        if w > MAX_DIM || h > MAX_DIM {
                            let ratio = (MAX_DIM as f32 / w.max(h) as f32).min(1.0);
                            let new_w = (w as f32 * ratio) as u32;
                            let new_h = (h as f32 * ratio) as u32;
                            let resized = img.resize_exact(
                                new_w,
                                new_h,
                                image::imageops::FilterType::Lanczos3,
                            );
                            let mut out: Vec<u8> = Vec::new();
                            let fmt = match ext {
                                "png" => ImageOutputFormat::Png,
                                "jpg" => ImageOutputFormat::Jpeg(85),
                                "gif" => ImageOutputFormat::Gif,
                                "webp" => ImageOutputFormat::WebP,
                                _ => ImageOutputFormat::Png,
                            };
                            if resized
                                .write_to(&mut std::io::Cursor::new(&mut out), fmt)
                                .is_ok()
                            {
                                final_bytes = out;
                            }
                        }
                    }
                }
            }

            let fname = format!(
                "{}-{}.{}",
                Uuid::new_v4(),
                chrono::Utc::now().timestamp(),
                ext
            );
            let mut path = PathBuf::from(base_dir);
            path.push(&fname);
            tokio::fs::write(&path, &final_bytes).await.map_err(|e| {
                AppError::from((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("write error: {}", e),
                ))
            })?;
            let url = format!("/uploads/{}", fname);
            uploaded_urls.push(url);
        }
    }

    Ok(Json(serde_json::json!({ "uploaded": uploaded_urls })))
}

pub fn router() -> Router {
    Router::new().route("/upload", post(upload_file))
}
