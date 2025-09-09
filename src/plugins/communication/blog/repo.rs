use crate::http_error::AppError;
use crate::plugins::communication::blog::models::BlogDto;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn insert_blog(
    pool: &PgPool,
    title: &str,
    slug: &str,
    body: &str,
    author: &str,
    is_active: bool,
    image_url: Option<&str>,
) -> Result<BlogDto, AppError> {
    let dto = sqlx::query_as::<_, BlogDto>("INSERT INTO blogs (title, slug, body, author, is_active, image_url) VALUES ($1,$2,$3,$4,$5,$6) RETURNING id, title, slug, body, author, is_active, image_url, created_at, updated_at")
        .bind(title)
        .bind(slug)
        .bind(body)
        .bind(author)
        .bind(is_active)
        .bind(image_url)
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;
    Ok(dto)
}

pub async fn get_blog(pool: &PgPool, id: Uuid) -> Result<BlogDto, AppError> {
    let dto = sqlx::query_as::<_, BlogDto>("SELECT id, title, slug, body, author, is_active, image_url, created_at, updated_at FROM blogs WHERE id = $1")
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;
    Ok(dto)
}

#[allow(dead_code)]
pub async fn list_blogs(
    pool: &PgPool,
    where_sql: &str,
    per_page: i64,
    offset: i64,
) -> Result<(Vec<BlogDto>, i64), AppError> {
    
    let items: Vec<BlogDto> = sqlx::query_as::<_, BlogDto>(where_sql)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(AppError::from)?;

    let total: i64 = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM blogs")
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;

    Ok((items, total))
}

pub async fn update_blog(
    pool: &PgPool,
    id: Uuid,
    title: Option<String>,
    slug: Option<String>,
    body: Option<String>,
    author: Option<String>,
    is_active: Option<bool>,
    image_url: Option<Option<String>>,
) -> Result<BlogDto, AppError> {
    let dto = sqlx::query_as::<_, BlogDto>("UPDATE blogs SET title = COALESCE($1, title), slug = COALESCE($2, slug), body = COALESCE($3, body), author = COALESCE($4, author), is_active = COALESCE($5, is_active), image_url = COALESCE($6, image_url), updated_at = now() WHERE id = $7 RETURNING id, title, slug, body, author, is_active, image_url, created_at, updated_at")
        .bind(title)
        .bind(slug)
        .bind(body)
        .bind(author)
        .bind(is_active)
        .bind(image_url)
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;
    Ok(dto)
}

pub async fn delete_blog(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
    sqlx::query("DELETE FROM blogs WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(AppError::from)?;
    Ok(())
}
