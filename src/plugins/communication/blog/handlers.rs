use axum::{extract::Path, Json, Extension};
use axum::http::StatusCode;
use serde_json::json;
use sqlx;
use crate::http_error::AppError;
use crate::plugins::communication::blog::models::{BlogCreate, BlogUpdate, BlogDto};
use sqlx::PgPool;

// Pagination params: page (1-based) and per_page
#[derive(Debug, serde::Deserialize)]
pub struct ListQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub is_active: Option<bool>,
    pub author: Option<String>,
}

pub async fn create_blog(Extension(pool): Extension<PgPool>, Json(payload): Json<BlogCreate>) -> Result<Json<BlogDto>, AppError> {
    let is_active = payload.is_active.unwrap_or(true);
    let dto: BlogDto = sqlx::query_as::<_, BlogDto>("INSERT INTO blogs (title, slug, body, author, is_active) VALUES ($1,$2,$3,$4,$5) RETURNING id, title, slug, body, author, is_active, created_at, updated_at")
        .bind(&payload.title)
        .bind(&payload.slug)
        .bind(&payload.body)
        .bind(&payload.author)
        .bind(is_active)
        .fetch_one(&pool)
        .await
        .map_err(AppError::from)?;

    Ok(Json(dto))
}

pub async fn get_blog(Extension(pool): Extension<PgPool>, Path(id): Path<uuid::Uuid>) -> Result<Json<BlogDto>, AppError> {
    let dto: BlogDto = sqlx::query_as::<_, BlogDto>("SELECT id, title, slug, body, author, is_active, created_at, updated_at FROM blogs WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .map_err(AppError::from)?;

    Ok(Json(dto))
}

pub async fn list_blogs(Extension(pool): Extension<PgPool>, axum::extract::Query(q): axum::extract::Query<ListQuery>) -> Result<Json<serde_json::Value>, AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let per_page = q.per_page.unwrap_or(20).clamp(1,100);
    let offset = ((page - 1) * per_page) as i64;
    // simple, explicit parameterized queries for common filter combinations
    let items: Vec<BlogDto> = match (q.is_active, q.author.clone()) {
        (None, None) => {
            sqlx::query_as::<_, BlogDto>("SELECT id, title, slug, body, author, is_active, created_at, updated_at FROM blogs WHERE 1=1 ORDER BY created_at DESC LIMIT $1 OFFSET $2")
                .bind(per_page as i64)
                .bind(offset)
                .fetch_all(&pool)
                .await
                .map_err(AppError::from)?
        }
        (Some(is_active), None) => {
            sqlx::query_as::<_, BlogDto>("SELECT id, title, slug, body, author, is_active, created_at, updated_at FROM blogs WHERE is_active = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3")
                .bind(is_active)
                .bind(per_page as i64)
                .bind(offset)
                .fetch_all(&pool)
                .await
                .map_err(AppError::from)?
        }
        (None, Some(author)) => {
            sqlx::query_as::<_, BlogDto>("SELECT id, title, slug, body, author, is_active, created_at, updated_at FROM blogs WHERE author = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3")
                .bind(author)
                .bind(per_page as i64)
                .bind(offset)
                .fetch_all(&pool)
                .await
                .map_err(AppError::from)?
        }
        (Some(is_active), Some(author)) => {
            sqlx::query_as::<_, BlogDto>("SELECT id, title, slug, body, author, is_active, created_at, updated_at FROM blogs WHERE is_active = $1 AND author = $2 ORDER BY created_at DESC LIMIT $3 OFFSET $4")
                .bind(is_active)
                .bind(author)
                .bind(per_page as i64)
                .bind(offset)
                .fetch_all(&pool)
                .await
                .map_err(AppError::from)?
        }
    };

    let total: i64 = match (q.is_active, q.author.clone()) {
        (None, None) => sqlx::query_scalar("SELECT COUNT(*) FROM blogs").fetch_one(&pool).await.map_err(AppError::from)?,
        (Some(is_active), None) => sqlx::query_scalar("SELECT COUNT(*) FROM blogs WHERE is_active = $1").bind(is_active).fetch_one(&pool).await.map_err(AppError::from)?,
        (None, Some(author)) => sqlx::query_scalar("SELECT COUNT(*) FROM blogs WHERE author = $1").bind(author).fetch_one(&pool).await.map_err(AppError::from)?,
        (Some(is_active), Some(author)) => sqlx::query_scalar("SELECT COUNT(*) FROM blogs WHERE is_active = $1 AND author = $2").bind(is_active).bind(author).fetch_one(&pool).await.map_err(AppError::from)?,
    };

    let fetched = items.len() as i64;
    let has_more = offset + fetched < total;
    Ok(Json(json!({ "items": items, "page": page, "per_page": per_page, "total": total, "has_more": has_more })))
}

pub async fn update_blog(Extension(pool): Extension<PgPool>, Path(id): Path<uuid::Uuid>, Json(payload): Json<BlogUpdate>) -> Result<Json<BlogDto>, AppError> {
    // simple update using COALESCE-like pattern
    let dto: BlogDto = sqlx::query_as::<_, BlogDto>("UPDATE blogs SET title = COALESCE($1, title), slug = COALESCE($2, slug), body = COALESCE($3, body), author = COALESCE($4, author), is_active = COALESCE($5, is_active), updated_at = now() WHERE id = $6 RETURNING id, title, slug, body, author, is_active, created_at, updated_at")
        .bind(payload.title)
        .bind(payload.slug)
        .bind(payload.body)
        .bind(payload.author)
        .bind(payload.is_active)
        .bind(id)
        .fetch_one(&pool)
        .await
        .map_err(AppError::from)?;

    Ok(Json(dto))
}

pub async fn delete_blog(Extension(pool): Extension<PgPool>, Path(id): Path<uuid::Uuid>) -> Result<StatusCode, AppError> {
    sqlx::query("DELETE FROM blogs WHERE id = $1").bind(id).execute(&pool).await.map_err(AppError::from)?;
    Ok(StatusCode::NO_CONTENT)
}
