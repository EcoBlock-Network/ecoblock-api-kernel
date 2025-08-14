use axum::{Json, extract::State};
use axum::http::StatusCode;
use crate::http_error::AppError;
use crate::plugins::auth::models::{LoginRequest, LoginResponse};
use sqlx::PgPool;
use bcrypt::verify;
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::Serialize;
use std::env;

#[derive(Serialize)]
struct Claims {
    sub: String,
    exp: usize,
}

pub async fn login(State(pool): State<PgPool>, Json(payload): Json<LoginRequest>) -> Result<Json<LoginResponse>, AppError> {
    if payload.username.is_empty() || payload.password.is_empty() {
        return Err(AppError::new(StatusCode::BAD_REQUEST, "username and password required").with_code("invalid_credentials"));
    }

    let row = sqlx::query!("SELECT id, username, password_hash FROM users WHERE username = $1", payload.username)
        .fetch_optional(&pool)
        .await
        .map_err(AppError::from)?;

    let row = match row {
        Some(r) => r,
        None => return Err(AppError::new(StatusCode::UNAUTHORIZED, "invalid username or password").with_code("invalid_credentials")),
    };

    let valid = verify(&payload.password, &row.password_hash).map_err(|e| AppError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if !valid {
        return Err(AppError::new(StatusCode::UNAUTHORIZED, "invalid username or password").with_code("invalid_credentials"));
    }

    // create JWT
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());
    let exp = (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize;
    let claims = Claims { sub: row.id.to_string(), exp };
    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes())).map_err(|e| AppError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(LoginResponse { token }))
}
