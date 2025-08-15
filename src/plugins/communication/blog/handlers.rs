use axum::{extract::Path, Json, Extension};
use axum::http::StatusCode;
use sqlx;
use crate::http_error::AppError;
use crate::plugins::communication::blog::models::{BlogCreate, BlogUpdate, BlogDto};
use crate::plugins::communication::shared::ListResponse;
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

pub async fn list_blogs(Extension(pool): Extension<PgPool>, axum::extract::Query(q): axum::extract::Query<ListQuery>) -> Result<Json<ListResponse<BlogDto>>, AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let per_page = q.per_page.unwrap_or(20).clamp(1,100);
    let offset = ((page - 1) * per_page) as i64;

    enum Param {
        Bool(bool),
        Str(String),
    }

    let mut where_clauses: Vec<String> = Vec::new();
    let mut params: Vec<Param> = Vec::new();
    if let Some(is_active) = q.is_active {
        where_clauses.push(format!("is_active = ${}", params.len() + 1));
        params.push(Param::Bool(is_active));
    }
    if let Some(author) = q.author.clone() {
        where_clauses.push(format!("author = ${}", params.len() + 1));
        params.push(Param::Str(author));
    }

    let where_sql = if where_clauses.is_empty() { "1=1".to_string() } else { where_clauses.join(" AND ") };

    let items_sql = format!("SELECT id, title, slug, body, author, is_active, created_at, updated_at FROM blogs WHERE {} ORDER BY created_at DESC LIMIT ${} OFFSET ${}", where_sql, params.len() + 1, params.len() + 2);
    let mut items_q = sqlx::query_as::<_, BlogDto>(&items_sql);
    for p in &params {
        match p {
            Param::Bool(b) => { items_q = items_q.bind(*b); }
            Param::Str(s) => { items_q = items_q.bind(s.clone()); }
        }
    }
    items_q = items_q.bind(per_page as i64).bind(offset);
    let items: Vec<BlogDto> = items_q.fetch_all(&pool).await.map_err(AppError::from)?;

    let count_sql = format!("SELECT COUNT(*) FROM blogs WHERE {}", where_sql);
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
