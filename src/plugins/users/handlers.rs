use axum::{Json, extract::Path};
use axum::http::StatusCode;
use sqlx::{PgPool, Row};
use bcrypt::{hash, DEFAULT_COST};
use crate::plugins::users::models::{UserDto, CreateUser, UpdateUser};
use uuid::Uuid;
use crate::http_error::AppError;

pub async fn create_user(pool: PgPool, payload: CreateUser) -> Result<Json<UserDto>, AppError> {
    if !payload.email.contains('@') {
        return Err(AppError::new(StatusCode::BAD_REQUEST, "invalid email"));
    }
    if payload.password.len() < 8 {
        return Err(AppError::new(StatusCode::BAD_REQUEST, "password too short"));
    }

    let password_hash = hash(&payload.password, DEFAULT_COST).map_err(|e| AppError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let row = sqlx::query("INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3) RETURNING id, username, email")
        .bind(&payload.username)
        .bind(&payload.email)
        .bind(&password_hash)
        .fetch_one(&pool)
        .await
        .map_err(AppError::from)?;

    let id: uuid::Uuid = row.get("id");
    let username: String = row.get("username");
    let email: String = row.get("email");

    Ok(Json(UserDto { id, username, email }))
}

pub async fn list_users(pool: PgPool) -> Result<Json<Vec<UserDto>>, AppError> {
    let rows = sqlx::query("SELECT id, username, email FROM users ORDER BY created_at DESC")
        .fetch_all(&pool)
        .await
        .map_err(AppError::from)?;

    let users = rows.into_iter().map(|r| UserDto { id: r.get("id"), username: r.get("username"), email: r.get("email") }).collect();
    Ok(Json(users))
}

pub async fn get_user(pool: PgPool, Path(id): Path<Uuid>) -> Result<Json<UserDto>, AppError> {
    let row = sqlx::query("SELECT id, username, email FROM users WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .map_err(AppError::from)?;

    Ok(Json(UserDto { id: row.get("id"), username: row.get("username"), email: row.get("email") }))
}
pub async fn update_user(pool: PgPool, Path(id): Path<Uuid>, Json(payload): Json<UpdateUser>) -> Result<Json<UserDto>, AppError> {
    let current = sqlx::query("SELECT username, email FROM users WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .map_err(AppError::from)?;

    let new_username = payload.username.unwrap_or(current.get("username"));
    let new_email = payload.email.unwrap_or(current.get("email"));

    let row = sqlx::query("UPDATE users SET username = $1, email = $2 WHERE id = $3 RETURNING id, username, email")
        .bind(new_username)
        .bind(new_email)
        .bind(id)
        .fetch_one(&pool)
        .await
        .map_err(AppError::from)?;

    Ok(Json(UserDto { id: row.get("id"), username: row.get("username"), email: row.get("email") }))
}

pub async fn delete_user(pool: PgPool, Path(id): Path<Uuid>) -> Result<StatusCode, AppError> {
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(AppError::from)?;

    Ok(StatusCode::NO_CONTENT)
}
