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

pub async fn insert_user_with_admin(pool: &PgPool, username: &str, email: &str, password: &str, is_admin: bool) -> Result<UserDto, AppError> {
    let password_hash = hash(password, DEFAULT_COST).map_err(|e| AppError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let row = sqlx::query("INSERT INTO users (username, email, password_hash, is_admin) VALUES ($1, $2, $3, $4) RETURNING id, username, email")
        .bind(username)
        .bind(email)
        .bind(&password_hash)
        .bind(is_admin)
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

pub async fn is_admin(pool: &PgPool, id: Uuid) -> Result<bool, AppError> {
    let row = sqlx::query_scalar::<_, bool>("SELECT is_admin FROM users WHERE id = $1").bind(id).fetch_one(pool).await.map_err(AppError::from)?;
    Ok(row)
}

pub async fn set_admin(pool: &PgPool, id: Uuid, admin: bool) -> Result<(), AppError> {
    sqlx::query("UPDATE users SET is_admin = $1 WHERE id = $2").bind(admin).bind(id).execute(pool).await.map_err(AppError::from)?;
    Ok(())
}
