use axum::{Json, extract::Path};
use axum::http::StatusCode;
use sqlx::PgPool;
use crate::plugins::users::models::{UserDto, CreateUser, UpdateUser};
use crate::plugins::auth::handlers::AuthUser;
use uuid::Uuid;
use crate::http_error::AppError;
use crate::plugins::users::repo as repo;

pub async fn create_user(pool: PgPool, payload: CreateUser) -> Result<Json<UserDto>, AppError> {
    if !payload.email.contains('@') {
        return Err(AppError::new(StatusCode::BAD_REQUEST, "invalidEmail"));
    }
    if payload.password.len() < 8 {
        return Err(AppError::new(StatusCode::BAD_REQUEST, "passwordTooShort"));
    }

    let dto = repo::insert_user(&pool, &payload.username, &payload.email, &payload.password).await?;
    Ok(Json(dto))
}

pub async fn list_users(pool: PgPool) -> Result<Json<Vec<UserDto>>, AppError> {
    let users = repo::list_users(&pool).await?;
    Ok(Json(users))
}

pub async fn get_user(pool: PgPool, Path(id): Path<Uuid>) -> Result<Json<UserDto>, AppError> {
    let user = repo::get_user(&pool, id).await?;
    Ok(Json(user))
}
pub async fn update_user(pool: PgPool, Path(id): Path<Uuid>, Json(payload): Json<UpdateUser>) -> Result<Json<UserDto>, AppError> {
    let current = repo::get_user(&pool, id).await?;
    let new_username = payload.username.unwrap_or(current.username);
    let new_email = payload.email.unwrap_or(current.email);
    let updated = repo::update_user(&pool, id, &new_username, &new_email).await?;
    Ok(Json(updated))
}

pub async fn delete_user(pool: PgPool, Path(id): Path<Uuid>) -> Result<StatusCode, AppError> {
    repo::delete_user(&pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn grant_admin(pool: PgPool, auth: AuthUser, Path(id): Path<Uuid>) -> Result<StatusCode, AppError> {
    let is_req_admin = repo::is_admin(&pool, auth.user_id).await?;
    if !is_req_admin {
        return Err(AppError::new(StatusCode::FORBIDDEN, "onlyAdminCanGrantAdmin"));
    }
    repo::set_admin(&pool, id, true).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn create_admin(pool: PgPool, auth: AuthUser, payload: CreateUser) -> Result<Json<UserDto>, AppError> {
    let is_req_admin = repo::is_admin(&pool, auth.user_id).await?;
    if !is_req_admin {
        return Err(AppError::new(StatusCode::FORBIDDEN, "onlyAdminCanCreateAdminAccounts"));
    }
    if !payload.email.contains('@') {
        return Err(AppError::new(StatusCode::BAD_REQUEST, "invalidEmail"));
    }
    if payload.password.len() < 8 {
        return Err(AppError::new(StatusCode::BAD_REQUEST, "passwordTooShort"));
    }
    let dto = repo::insert_user_with_admin(&pool, &payload.username, &payload.email, &payload.password, true).await?;
    Ok(Json(dto))
}
