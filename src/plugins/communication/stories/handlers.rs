use axum::{extract::Path, Json, Extension};
use axum::http::StatusCode;
use serde_json::json;
use sqlx;
use crate::http_error::AppError;
use crate::plugins::communication::stories::models::{StoryCreate, StoryUpdate, StoryDto};
use sqlx::PgPool;

#[derive(Debug, serde::Deserialize)]
pub struct ListQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub is_active: Option<bool>,
    pub created_by: Option<String>,
}

pub async fn create_story(Extension(pool): Extension<PgPool>, Json(payload): Json<StoryCreate>,) -> Result<Json<StoryDto>, AppError> {
    let is_active = payload.is_active.unwrap_or(true);
    let dto: StoryDto = sqlx::query_as::<_, StoryDto>("INSERT INTO stories (title, media_url, caption, is_active, expires_at, created_by) VALUES ($1,$2,$3,$4,$5,$6) RETURNING id, title, media_url, caption, is_active, created_at, expires_at, created_by")
        .bind(&payload.title)
        .bind(&payload.media_url)
        .bind(&payload.caption)
        .bind(is_active)
        .bind(payload.expires_at)
        .bind("admin")
        .fetch_one(&pool)
        .await
        .map_err(AppError::from)?;

    Ok(Json(dto))
}

pub async fn get_story(Extension(pool): Extension<PgPool>, Path(id): Path<uuid::Uuid>) -> Result<Json<StoryDto>, AppError> {
    let dto: StoryDto = sqlx::query_as::<_, StoryDto>("SELECT id, title, media_url, caption, is_active, created_at, expires_at, created_by FROM stories WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .map_err(AppError::from)?;

    Ok(Json(dto))
}

pub async fn list_stories(Extension(pool): Extension<PgPool>, axum::extract::Query(q): axum::extract::Query<ListQuery>) -> Result<Json<serde_json::Value>, AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let per_page = q.per_page.unwrap_or(20).clamp(1,100);
    let offset = ((page - 1) * per_page) as i64;

    let mut filters = String::new();
    if let Some(is_active) = q.is_active {
        filters.push_str(&format!(" AND is_active = {}", if is_active { "true" } else { "false" }));
    }
    // parameterized queries for safety
    let rows = match (&q.is_active, &q.created_by) {
        (None, None) => sqlx::query("SELECT id, title, media_url, caption, is_active, created_at, expires_at, created_by FROM stories WHERE 1=1 ORDER BY created_at DESC LIMIT $1 OFFSET $2")
            .bind(per_page as i64)
            .bind(offset)
            .fetch_all(&pool)
            .await
            .map_err(AppError::from)?,
        (Some(is_active), None) => sqlx::query("SELECT id, title, media_url, caption, is_active, created_at, expires_at, created_by FROM stories WHERE is_active = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3")
            .bind(is_active)
            .bind(per_page as i64)
            .bind(offset)
            .fetch_all(&pool)
            .await
            .map_err(AppError::from)?,
        (None, Some(created_by)) => sqlx::query("SELECT id, title, media_url, caption, is_active, created_at, expires_at, created_by FROM stories WHERE created_by = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3")
            .bind(created_by)
            .bind(per_page as i64)
            .bind(offset)
            .fetch_all(&pool)
            .await
            .map_err(AppError::from)?,
        (Some(is_active), Some(created_by)) => sqlx::query("SELECT id, title, media_url, caption, is_active, created_at, expires_at, created_by FROM stories WHERE is_active = $1 AND created_by = $2 ORDER BY created_at DESC LIMIT $3 OFFSET $4")
            .bind(is_active)
            .bind(created_by)
            .bind(per_page as i64)
            .bind(offset)
            .fetch_all(&pool)
            .await
            .map_err(AppError::from)?,
    };

    let items: Vec<StoryDto> = match (&q.is_active, &q.created_by) {
        (None, None) => sqlx::query_as::<_, StoryDto>("SELECT id, title, media_url, caption, is_active, created_at, expires_at, created_by FROM stories WHERE 1=1 ORDER BY created_at DESC LIMIT $1 OFFSET $2")
            .bind(per_page as i64)
            .bind(offset)
            .fetch_all(&pool)
            .await
            .map_err(AppError::from)?,
        (Some(is_active), None) => sqlx::query_as::<_, StoryDto>("SELECT id, title, media_url, caption, is_active, created_at, expires_at, created_by FROM stories WHERE is_active = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3")
            .bind(is_active)
            .bind(per_page as i64)
            .bind(offset)
            .fetch_all(&pool)
            .await
            .map_err(AppError::from)?,
        (None, Some(created_by)) => sqlx::query_as::<_, StoryDto>("SELECT id, title, media_url, caption, is_active, created_at, expires_at, created_by FROM stories WHERE created_by = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3")
            .bind(created_by)
            .bind(per_page as i64)
            .bind(offset)
            .fetch_all(&pool)
            .await
            .map_err(AppError::from)?,
        (Some(is_active), Some(created_by)) => sqlx::query_as::<_, StoryDto>("SELECT id, title, media_url, caption, is_active, created_at, expires_at, created_by FROM stories WHERE is_active = $1 AND created_by = $2 ORDER BY created_at DESC LIMIT $3 OFFSET $4")
            .bind(is_active)
            .bind(created_by)
            .bind(per_page as i64)
            .bind(offset)
            .fetch_all(&pool)
            .await
            .map_err(AppError::from)?,
    };

    let total: i64 = match (&q.is_active, &q.created_by) {
        (None, None) => sqlx::query_scalar("SELECT COUNT(*) FROM stories").fetch_one(&pool).await.map_err(AppError::from)?,
        (Some(is_active), None) => sqlx::query_scalar("SELECT COUNT(*) FROM stories WHERE is_active = $1").bind(*is_active).fetch_one(&pool).await.map_err(AppError::from)?,
        (None, Some(created_by)) => sqlx::query_scalar("SELECT COUNT(*) FROM stories WHERE created_by = $1").bind(created_by).fetch_one(&pool).await.map_err(AppError::from)?,
        (Some(is_active), Some(created_by)) => sqlx::query_scalar("SELECT COUNT(*) FROM stories WHERE is_active = $1 AND created_by = $2").bind(*is_active).bind(created_by).fetch_one(&pool).await.map_err(AppError::from)?,
    };

    Ok(Json(json!({ "items": items, "page": page, "per_page": per_page, "total": total })))
}

pub async fn update_story(Extension(pool): Extension<PgPool>, Path(id): Path<uuid::Uuid>, Json(payload): Json<StoryUpdate>) -> Result<Json<StoryDto>, AppError> {
    let dto: StoryDto = sqlx::query_as::<_, StoryDto>("UPDATE stories SET title = COALESCE($1, title), media_url = COALESCE($2, media_url), caption = COALESCE($3, caption), is_active = COALESCE($4, is_active), expires_at = COALESCE($5, expires_at) WHERE id = $6 RETURNING id, title, media_url, caption, is_active, created_at, expires_at, created_by")
        .bind(payload.title)
        .bind(payload.media_url)
        .bind(payload.caption)
        .bind(payload.is_active)
        .bind(payload.expires_at)
        .bind(id)
        .fetch_one(&pool)
        .await
        .map_err(AppError::from)?;

    Ok(Json(dto))
}

pub async fn delete_story(Extension(pool): Extension<PgPool>, Path(id): Path<uuid::Uuid>) -> Result<StatusCode, AppError> {
    sqlx::query("DELETE FROM stories WHERE id = $1").bind(id).execute(&pool).await.map_err(AppError::from)?;
    Ok(StatusCode::NO_CONTENT)
}
