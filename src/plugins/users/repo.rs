use sqlx::PgPool;
use sqlx::Row;
use axum::http::StatusCode;
use crate::http_error::AppError;
use crate::plugins::users::models::UserDto;
use uuid::Uuid;
use bcrypt::{hash, DEFAULT_COST};

pub async fn insert_user(pool: &PgPool, username: &str, email: &str, password: &str) -> Result<UserDto, AppError> {
    let password_hash = hash(password, DEFAULT_COST).map_err(|e| AppError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let row = sqlx::query("INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3) RETURNING id, username, email")
        .bind(username)
        .bind(email)
        .bind(&password_hash)
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;

    Ok(UserDto { id: row.get("id"), username: row.get("username"), email: row.get("email") })
}

pub async fn list_users(pool: &PgPool) -> Result<Vec<UserDto>, AppError> {
    let rows = sqlx::query("SELECT id, username, email FROM users ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

    let users = rows.into_iter().map(|r| UserDto { id: r.get("id"), username: r.get("username"), email: r.get("email") }).collect();
    Ok(users)
}

pub async fn get_user(pool: &PgPool, id: Uuid) -> Result<UserDto, AppError> {
    let row = sqlx::query("SELECT id, username, email FROM users WHERE id = $1")
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;

    Ok(UserDto { id: row.get("id"), username: row.get("username"), email: row.get("email") })
}

pub async fn update_user(pool: &PgPool, id: Uuid, username: &str, email: &str) -> Result<UserDto, AppError> {
    let row = sqlx::query("UPDATE users SET username = $1, email = $2 WHERE id = $3 RETURNING id, username, email")
        .bind(username)
        .bind(email)
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;

    Ok(UserDto { id: row.get("id"), username: row.get("username"), email: row.get("email") })
}

pub async fn delete_user(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
    sqlx::query("DELETE FROM users WHERE id = $1").bind(id).execute(pool).await.map_err(AppError::from)?;
    Ok(())
}
