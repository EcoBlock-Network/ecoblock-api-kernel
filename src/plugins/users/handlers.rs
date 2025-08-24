use crate::cache::DynCache;
use crate::http_error::AppError;
use crate::plugins::auth::handlers::AuthUser;
use crate::plugins::users::models::{CreateUser, UpdateUser, UserDto};
use crate::plugins::users::repo;
use axum::http::StatusCode;
use axum::{Extension, Json, extract::Path};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_user(pool: PgPool, payload: CreateUser) -> Result<Json<UserDto>, AppError> {
    if !payload.email.contains('@') {
        return Err(AppError::new(StatusCode::BAD_REQUEST, "invalidEmail"));
    }
    if payload.password.len() < 8 {
        return Err(AppError::new(StatusCode::BAD_REQUEST, "passwordTooShort"));
    }

    let dto =
        repo::insert_user(&pool, &payload.username, &payload.email, &payload.password).await?;
    Ok(Json(dto))
}

pub async fn list_users(pool: PgPool) -> Result<Json<Vec<UserDto>>, AppError> {
    let users = repo::list_users(&pool).await?;
    Ok(Json(users))
}

pub async fn get_user(
    Extension(cache_opt): Extension<Option<DynCache>>,
    pool: PgPool,
    Path(id): Path<Uuid>,
) -> Result<Json<UserDto>, AppError> {
    // try cache first
    if let Some(cache) = cache_opt.as_ref().map(|c| c.clone()) {
        let key = format!("user:{}", id);
        let maybe_bytes_res = cache.get(&key).await;
        if let Ok(maybe_bytes) = maybe_bytes_res {
            if let Some(bytes_vec) = maybe_bytes {
                if let Ok(dto) = serde_json::from_slice::<UserDto>(&bytes_vec) {
                    return Ok(Json(dto));
                }
            }
        }
    }
    let user = repo::get_user(&pool, id).await?;
    if let Some(cache) = cache_opt.as_ref().map(|c| c.clone()) {
        let key = format!("user:{}", id);
        if let Ok(b) = serde_json::to_vec(&user) {
            let _ = cache.set(&key, b, None).await;
        }
    }
    Ok(Json(user))
}
pub async fn update_user(
    Extension(cache_opt): Extension<Option<DynCache>>,
    pool: PgPool,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateUser>,
) -> Result<Json<UserDto>, AppError> {
    let current = repo::get_user(&pool, id).await?;
    let new_username = payload.username.unwrap_or(current.username);
    let new_email = payload.email.unwrap_or(current.email);
    let updated = repo::update_user(&pool, id, &new_username, &new_email).await?;
    // invalidate cache
    if let Some(cache) = cache_opt.as_ref().map(|c| c.clone()) {
        let key = format!("user:{}", id);
        let _ = cache.delete(&key).await;
    }
    Ok(Json(updated))
}

pub async fn delete_user(
    Extension(cache_opt): Extension<Option<DynCache>>,
    pool: PgPool,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    repo::delete_user(&pool, id).await?;
    if let Some(cache) = cache_opt.as_ref().map(|c| c.clone()) {
        let key = format!("user:{}", id);
        let _ = cache.delete(&key).await;
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn grant_admin(
    Extension(cache_opt): Extension<Option<DynCache>>,
    pool: PgPool,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let is_req_admin = repo::is_admin(&pool, auth.user_id).await?;
    if !is_req_admin {
        return Err(AppError::new(
            StatusCode::FORBIDDEN,
            "onlyAdminCanGrantAdmin",
        ));
    }
    repo::set_admin(&pool, id, true).await?;
    if let Some(cache) = cache_opt.as_ref().map(|c| c.clone()) {
        let key = format!("user:{}", id);
        let _ = cache.delete(&key).await;
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn create_admin(
    Extension(_cache_opt): Extension<Option<DynCache>>,
    pool: PgPool,
    auth: AuthUser,
    payload: CreateUser,
) -> Result<Json<UserDto>, AppError> {
    let is_req_admin = repo::is_admin(&pool, auth.user_id).await?;
    if !is_req_admin {
        return Err(AppError::new(
            StatusCode::FORBIDDEN,
            "onlyAdminCanCreateAdminAccounts",
        ));
    }
    if !payload.email.contains('@') {
        return Err(AppError::new(StatusCode::BAD_REQUEST, "invalidEmail"));
    }
    if payload.password.len() < 8 {
        return Err(AppError::new(StatusCode::BAD_REQUEST, "passwordTooShort"));
    }
    let dto = repo::insert_user_with_admin(
        &pool,
        &payload.username,
        &payload.email,
        &payload.password,
        true,
    )
    .await?;
    // We can't invalidate by id here since we don't know it yet; caller can fetch and cache after creation.
    Ok(Json(dto))
}
