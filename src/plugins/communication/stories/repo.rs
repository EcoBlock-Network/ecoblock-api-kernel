use crate::http_error::AppError;
use crate::plugins::communication::stories::models::StoryDto;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn insert_story(
    pool: &PgPool,
    title: &str,
    media_url: &str,
    caption: &str,
    is_active: bool,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
    created_by: &str,
) -> Result<StoryDto, AppError> {
    let dto = sqlx::query_as::<_, StoryDto>("INSERT INTO stories (title, media_url, caption, is_active, expires_at, created_by) VALUES ($1,$2,$3,$4,$5,$6) RETURNING id, title, media_url, caption, is_active, created_at, expires_at, created_by")
        .bind(title)
        .bind(media_url)
        .bind(caption)
        .bind(is_active)
        .bind(expires_at)
        .bind(created_by)
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;
    Ok(dto)
}

pub async fn get_story(pool: &PgPool, id: Uuid) -> Result<StoryDto, AppError> {
    let dto = sqlx::query_as::<_, StoryDto>("SELECT id, title, media_url, caption, is_active, created_at, expires_at, created_by FROM stories WHERE id = $1")
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;
    Ok(dto)
}

#[allow(dead_code)]
pub async fn list_stories(
    pool: &PgPool,
    where_sql: &str,
    per_page: i64,
    offset: i64,
) -> Result<(Vec<StoryDto>, i64), AppError> {
    let items: Vec<StoryDto> = sqlx::query_as::<_, StoryDto>(where_sql)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

    let total: i64 = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM stories")
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;

    Ok((items, total))
}

pub async fn update_story(
    pool: &PgPool,
    id: Uuid,
    title: Option<String>,
    media_url: Option<String>,
    caption: Option<String>,
    is_active: Option<bool>,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
) -> Result<StoryDto, AppError> {
    let dto = sqlx::query_as::<_, StoryDto>("UPDATE stories SET title = COALESCE($1, title), media_url = COALESCE($2, media_url), caption = COALESCE($3, caption), is_active = COALESCE($4, is_active), expires_at = COALESCE($5, expires_at) WHERE id = $6 RETURNING id, title, media_url, caption, is_active, created_at, expires_at, created_by")
        .bind(title)
        .bind(media_url)
        .bind(caption)
        .bind(is_active)
        .bind(expires_at)
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;
    Ok(dto)
}

pub async fn delete_story(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
    sqlx::query("DELETE FROM stories WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(AppError::from)?;
    Ok(())
}
