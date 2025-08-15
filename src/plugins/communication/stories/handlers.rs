use axum::{extract::Path, Json, Extension};
use axum::http::StatusCode;
use sqlx;
use crate::http_error::AppError;
use crate::plugins::communication::stories::models::{StoryCreate, StoryUpdate, StoryDto};
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
