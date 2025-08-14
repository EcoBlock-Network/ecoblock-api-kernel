use axum::{Router, routing::{post, get, put, delete}, Json, extract::Path};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use crate::kernel::Plugin;
use uuid::Uuid;
use bcrypt::{hash, DEFAULT_COST};

#[derive(Serialize)]
struct UserDto {
    id: Uuid,
    username: String,
    email: String,
}

#[derive(Deserialize)]
struct CreateUser {
    username: String,
    email: String,
    password: String,
}

#[derive(Deserialize)]
struct UpdateUser {
    username: Option<String>,
    email: Option<String>,
}

pub struct UsersPlugin {
    pub pool: PgPool,
}

async fn create_user(pool: PgPool, payload: CreateUser) -> Result<Json<UserDto>, (axum::http::StatusCode, String)> {
    let password_hash = hash(&payload.password, DEFAULT_COST).map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let row = sqlx::query!(
        r#"INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3) RETURNING id, username, email"#,
        payload.username,
        payload.email,
        password_hash
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(UserDto { id: row.id, username: row.username, email: row.email }))
}

impl UsersPlugin {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

async fn list_users(pool: PgPool) -> Result<Json<Vec<UserDto>>, (StatusCode, String)> {
    let rows = sqlx::query!("SELECT id, username, email FROM users ORDER BY created_at DESC")
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let users = rows.into_iter().map(|r| UserDto { id: r.id, username: r.username, email: r.email }).collect();
    Ok(Json(users))
}

async fn get_user(pool: PgPool, Path(id): Path<Uuid>) -> Result<Json<UserDto>, (StatusCode, String)> {
    let row = sqlx::query!("SELECT id, username, email FROM users WHERE id = $1", id)
        .fetch_one(&pool)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

    Ok(Json(UserDto { id: row.id, username: row.username, email: row.email }))
}

async fn update_user(pool: PgPool, Path(id): Path<Uuid>, Json(payload): Json<UpdateUser>) -> Result<Json<UserDto>, (StatusCode, String)> {
    // build set clause dynamically (simple approach)
    let current = sqlx::query!("SELECT username, email FROM users WHERE id = $1", id)
        .fetch_one(&pool)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

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
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(UserDto { id: row.id, username: row.username, email: row.email }))
}

async fn delete_user(pool: PgPool, Path(id): Path<Uuid>) -> Result<StatusCode, (StatusCode, String)> {
    sqlx::query!("DELETE FROM users WHERE id = $1", id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

#[async_trait::async_trait]
impl Plugin for UsersPlugin {
    async fn router(&self) -> Router {
        // create separate clones so each closure takes ownership of its clone
        let p_create = self.pool.clone();
        let p_list = self.pool.clone();
        let p_get = self.pool.clone();
        let p_update = self.pool.clone();
        let p_delete = self.pool.clone();

        Router::new()
            .route("/", post(move |Json(payload): Json<CreateUser>| {
                let pool = p_create.clone();
                async move { create_user(pool, payload).await }
            }))
            .route("/", get(move || {
                let pool = p_list.clone();
                async move { list_users(pool).await }
            }))
            .route("/:id", get(move |Path(id): Path<Uuid>| {
                let pool = p_get.clone();
                async move { get_user(pool, Path(id)).await }
            }))
            .route("/:id", put(move |Path(id): Path<Uuid>, Json(payload): Json<UpdateUser>| {
                let pool = p_update.clone();
                async move { update_user(pool, Path(id), Json(payload)).await }
            }))
            .route("/:id", delete(move |Path(id): Path<Uuid>| {
                let pool = p_delete.clone();
                async move { delete_user(pool, Path(id)).await }
            }))
    }

    fn name(&self) -> &'static str {
        "users"
    }
}
