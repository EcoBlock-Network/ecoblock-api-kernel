use axum::{Json, extract::State};
use axum::http::StatusCode;
use crate::http_error::AppError;
use crate::plugins::auth::models::{LoginRequest, LoginResponse};
use sqlx::{PgPool, Row};
use bcrypt::verify;
use jsonwebtoken::{EncodingKey, Header, encode, DecodingKey, Validation, decode};
use serde::Serialize;
use std::env;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use async_trait::async_trait;
use crate::plugins::users::models::UserDto;

#[derive(Serialize, serde::Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

#[derive(Clone)]
pub struct AuthUser {
    pub user_id: uuid::Uuid,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_hdr = parts
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| AppError::new(StatusCode::UNAUTHORIZED, "missing authorization").with_code("missing_token"))?;

        if !auth_hdr.starts_with("Bearer ") {
            return Err(AppError::new(StatusCode::UNAUTHORIZED, "invalid authorization header").with_code("invalid_token"));
        }
        let token = &auth_hdr[7..];
        let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());
        let validation = Validation::default();
        let token_data = decode::<Claims>(token, &DecodingKey::from_secret(secret.as_bytes()), &validation)
            .map_err(|_| AppError::new(StatusCode::UNAUTHORIZED, "invalid token").with_code("invalid_token"))?;
        let sub = token_data.claims.sub;
        let user_id = uuid::Uuid::parse_str(&sub).map_err(|_| AppError::new(StatusCode::UNAUTHORIZED, "invalid token subject").with_code("invalid_token"))?;
        Ok(AuthUser { user_id })
    }
}

pub async fn login(State(pool): State<PgPool>, Json(payload): Json<LoginRequest>) -> Result<Json<LoginResponse>, AppError> {
    if payload.username.is_empty() || payload.password.is_empty() {
        return Err(AppError::new(StatusCode::BAD_REQUEST, "username and password required").with_code("invalid_credentials"));
    }

    let opt_row = sqlx::query("SELECT id, username, password_hash FROM users WHERE username = $1")
        .bind(&payload.username)
        .fetch_optional(&pool)
        .await
        .map_err(AppError::from)?;

    let row = match opt_row {
        Some(r) => r,
        None => return Err(AppError::new(StatusCode::UNAUTHORIZED, "invalid username or password").with_code("invalid_credentials")),
    };

    let password_hash: String = row.get("password_hash");
    let id: uuid::Uuid = row.get("id");

    let valid = verify(&payload.password, &password_hash).map_err(|e| AppError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if !valid {
        return Err(AppError::new(StatusCode::UNAUTHORIZED, "invalid username or password").with_code("invalid_credentials"));
    }

    
    let secret = env::var("JWT_SECRET").map_err(|_| AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "JWT_SECRET not configured").with_code("config_error"))?;
    let exp = (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize;
    let claims = Claims { sub: id.to_string(), exp };
    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes())).map_err(|e| AppError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(LoginResponse { token }))
}

pub async fn whoami(State(pool): State<PgPool>, auth: AuthUser) -> Result<Json<UserDto>, AppError> {
    let row = sqlx::query("SELECT id, username, email FROM users WHERE id = $1")
        .bind(auth.user_id)
        .fetch_one(&pool)
        .await
        .map_err(AppError::from)?;

    let user = UserDto { id: row.get("id"), username: row.get("username"), email: row.get("email") };
    Ok(Json(user))
}
