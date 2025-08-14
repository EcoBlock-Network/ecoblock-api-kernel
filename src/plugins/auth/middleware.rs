use axum::http::{Request, StatusCode};
use axum::response::Response;
use axum::middleware::Next;
use axum::body::Body;
use crate::http_error::AppError;
use jsonwebtoken::{DecodingKey, Validation, decode};
use std::env;
use serde::Deserialize;

#[derive(Deserialize)]
struct ClaimsLite {
    sub: String,
}

pub async fn require_auth(mut req: Request<Body>, next: Next) -> Result<Response, AppError> {
    let auth_hdr = req.headers().get("authorization").and_then(|v| v.to_str().ok()).ok_or_else(|| AppError::new(StatusCode::UNAUTHORIZED, "missing authorization").with_code("missing_token"))?;
    if !auth_hdr.starts_with("Bearer ") {
        return Err(AppError::new(StatusCode::UNAUTHORIZED, "invalid authorization header").with_code("invalid_token"));
    }
    let token = &auth_hdr[7..];
    let secret = env::var("JWT_SECRET").map_err(|_| AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "JWT_SECRET not configured").with_code("config_error"))?;
    let validation = Validation::default();
    let token_data = decode::<ClaimsLite>(token, &DecodingKey::from_secret(secret.as_bytes()), &validation)
        .map_err(|_| AppError::new(StatusCode::UNAUTHORIZED, "invalid token").with_code("invalid_token"))?;
    let user_id = uuid::Uuid::parse_str(&token_data.claims.sub).map_err(|_| AppError::new(StatusCode::UNAUTHORIZED, "invalid token subject").with_code("invalid_token"))?;
    // insert into extensions for handlers to use
    req.extensions_mut().insert(super::handlers::AuthUser { user_id });
    Ok(next.run(req).await)
}
