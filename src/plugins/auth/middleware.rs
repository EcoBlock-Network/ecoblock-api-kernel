use crate::http_error::AppError;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use jsonwebtoken::{DecodingKey, Validation, decode};
use serde::Deserialize;
use std::env;

#[derive(Deserialize)]
struct ClaimsLite {
    sub: String,
}

pub async fn require_auth(mut req: Request<Body>, next: Next) -> Result<Response, AppError> {
    let auth_hdr = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            AppError::new(StatusCode::UNAUTHORIZED, "missingAuthorization")
                .with_code("missing_token")
        })?;
    if !auth_hdr.starts_with("Bearer ") {
        return Err(
            AppError::new(StatusCode::UNAUTHORIZED, "invalidAuthorizationHeader")
                .with_code("invalid_token"),
        );
    }
    let token = &auth_hdr[7..];
    let secret = env::var("JWT_SECRET").map_err(|_| {
        AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "jwtSecretNotConfigured")
            .with_code("config_error")
    })?;
    let validation = Validation::default();
    let token_data = decode::<ClaimsLite>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|_| {
        AppError::new(StatusCode::UNAUTHORIZED, "invalidToken").with_code("invalid_token")
    })?;
    let user_id = uuid::Uuid::parse_str(&token_data.claims.sub).map_err(|_| {
        AppError::new(StatusCode::UNAUTHORIZED, "invalidTokenSubject").with_code("invalid_token")
    })?;

    req.extensions_mut()
        .insert(super::handlers::AuthUser { user_id });
    Ok(next.run(req).await)
}
