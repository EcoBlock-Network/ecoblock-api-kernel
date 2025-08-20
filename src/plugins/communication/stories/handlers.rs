use axum::{extract::Path, Json, Extension};
use axum::http::StatusCode;
use sqlx;
use crate::http_error::AppError;
use crate::plugins::communication::stories::models::{StoryCreate, StoryUpdate, StoryDto};
use crate::plugins::communication::stories::repo as repo;
use crate::plugins::communication::shared::ListResponse;
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
    let dto = repo::insert_story(&pool, payload.title.as_deref().unwrap_or(""), &payload.media_url, payload.caption.as_deref().unwrap_or(""), is_active, payload.expires_at, "admin").await?;
    Ok(Json(dto))
}

pub async fn get_story(Extension(pool): Extension<PgPool>, Path(id): Path<uuid::Uuid>) -> Result<Json<StoryDto>, AppError> {
    let dto = repo::get_story(&pool, id).await?;
    Ok(Json(dto))
}

pub async fn list_stories(Extension(pool): Extension<PgPool>, axum::extract::Query(q): axum::extract::Query<ListQuery>) -> Result<Json<ListResponse<StoryDto>>, AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let per_page = q.per_page.unwrap_or(20).clamp(1,100);
    let offset = ((page - 1) * per_page) as i64;

    enum Param { Bool(bool), Str(String) }
    let mut where_clauses: Vec<String> = Vec::new();
    let mut params: Vec<Param> = Vec::new();
    if let Some(is_active) = q.is_active {
        where_clauses.push(format!("is_active = ${}", params.len() + 1));
        params.push(Param::Bool(is_active));
    }
    if let Some(created_by) = q.created_by.clone() {
        where_clauses.push(format!("created_by = ${}", params.len() + 1));
        params.push(Param::Str(created_by));
    }

    let where_sql = if where_clauses.is_empty() { "1=1".to_string() } else { where_clauses.join(" AND ") };

    let items_sql = format!("SELECT id, title, media_url, caption, is_active, created_at, expires_at, created_by FROM stories WHERE {} ORDER BY created_at DESC LIMIT ${} OFFSET ${}", where_sql, params.len() + 1, params.len() + 2);
    let mut items_q = sqlx::query_as::<_, StoryDto>(&items_sql);
    for p in &params {
        match p {
            Param::Bool(b) => { items_q = items_q.bind(*b); }
            Param::Str(s) => { items_q = items_q.bind(s.clone()); }
        }
    }
    items_q = items_q.bind(per_page as i64).bind(offset);
    let items: Vec<StoryDto> = items_q.fetch_all(&pool).await.map_err(AppError::from)?;

    let count_sql = format!("SELECT COUNT(*) FROM stories WHERE {}", where_sql);
    let mut count_q = sqlx::query_scalar::<_, i64>(&count_sql);
    for p in &params {
        match p {
            Param::Bool(b) => { count_q = count_q.bind(*b); }
            Param::Str(s) => { count_q = count_q.bind(s.clone()); }
        }
    }
    let total: i64 = count_q.fetch_one(&pool).await.map_err(AppError::from)?;

    let fetched = items.len() as i64;
    let has_more = offset + fetched < total;
    let total_pages = if total == 0 { 0 } else { ((total as f64) / (per_page as f64)).ceil() as i64 };
    let resp = ListResponse { items, page, per_page, total, total_pages, has_more };
    Ok(Json(resp))
}

pub async fn update_story(Extension(pool): Extension<PgPool>, Path(id): Path<uuid::Uuid>, Json(payload): Json<StoryUpdate>) -> Result<Json<StoryDto>, AppError> {
    let dto = repo::update_story(&pool, id, payload.title, payload.media_url, payload.caption, payload.is_active, payload.expires_at).await?;
    Ok(Json(dto))
}

pub async fn delete_story(Extension(pool): Extension<PgPool>, Path(id): Path<uuid::Uuid>) -> Result<StatusCode, AppError> {
    repo::delete_story(&pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
