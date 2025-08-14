use axum::{Json, extract::Path};
use axum::http::StatusCode;
use sqlx::{PgPool, Error as SqlxError};
use bcrypt::{hash, DEFAULT_COST};
use crate::plugins::users::models::{UserDto, CreateUser, UpdateUser};
use uuid::Uuid;

fn db_error_to_response(e: SqlxError) -> (StatusCode, String) {
    match e {
        SqlxError::Database(db_err) => {
            // Postgres unique violation
            if let Some(code) = db_err.code() {
                if code == "23505" {
                    return (StatusCode::CONFLICT, "duplicate key".to_string());
                }
            }
            (StatusCode::INTERNAL_SERVER_ERROR, db_err.message().to_string())
        }
        other => (StatusCode::INTERNAL_SERVER_ERROR, other.to_string()),
    }
}

pub async fn create_user(pool: PgPool, payload: CreateUser) -> Result<Json<UserDto>, (StatusCode, String)> {
    if !payload.email.contains('@') {
        return Err((StatusCode::BAD_REQUEST, "invalid email".to_string()));
    }
    if payload.password.len() < 8 {
        return Err((StatusCode::BAD_REQUEST, "password too short".to_string()));
    }

    let password_hash = hash(&payload.password, DEFAULT_COST).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let row = sqlx::query!(
        r#"INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3) RETURNING id, username, email"#,
        payload.username,
        payload.email,
        password_hash
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| db_error_to_response(e))?;

    Ok(Json(UserDto { id: row.id, username: row.username, email: row.email }))
}

pub async fn list_users(pool: PgPool) -> Result<Json<Vec<UserDto>>, (StatusCode, String)> {
    let rows = sqlx::query!("SELECT id, username, email FROM users ORDER BY created_at DESC")
        .fetch_all(&pool)
        .await
        .map_err(|e| db_error_to_response(e))?;

    let users = rows.into_iter().map(|r| UserDto { id: r.id, username: r.username, email: r.email }).collect();
    Ok(Json(users))
}

pub async fn get_user(pool: PgPool, Path(id): Path<Uuid>) -> Result<Json<UserDto>, (StatusCode, String)> {
    let row = sqlx::query!("SELECT id, username, email FROM users WHERE id = $1", id)
        .fetch_one(&pool)
        .await
        .map_err(|e| db_error_to_response(e))?;

    Ok(Json(UserDto { id: row.id, username: row.username, email: row.email }))
}

pub async fn update_user(pool: PgPool, Path(id): Path<Uuid>, Json(payload): Json<UpdateUser>) -> Result<Json<UserDto>, (StatusCode, String)> {
    let current = sqlx::query!("SELECT username, email FROM users WHERE id = $1", id)
        .fetch_one(&pool)
        .await
        .map_err(|e| db_error_to_response(e))?;

    let new_username = payload.username.unwrap_or(current.username);
    let new_email = payload.email.unwrap_or(current.email);

    let row = sqlx::query!(
        "UPDATE users SET username = $1, email = $2 WHERE id = $3 RETURNING id, username, email",
        new_username,
        new_email,
        id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| db_error_to_response(e))?;

    Ok(Json(UserDto { id: row.id, username: row.username, email: row.email }))
}

pub async fn delete_user(pool: PgPool, Path(id): Path<Uuid>) -> Result<StatusCode, (StatusCode, String)> {
    sqlx::query!("DELETE FROM users WHERE id = $1", id)
        .execute(&pool)
        .await
        .map_err(|e| db_error_to_response(e))?;

    Ok(StatusCode::NO_CONTENT)
}
