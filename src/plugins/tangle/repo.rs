use sqlx::PgPool;
use crate::http_error::AppError;
use crate::plugins::tangle::models::TangleBlockRow;
use serde_json::Value;
use uuid::Uuid;

pub async fn insert_block(
    pool: &PgPool,
    id: Option<Uuid>,
    parents: &Vec<String>,
    data: &Value,
    signature: &[u8],
    public_key: &str,
) -> Result<TangleBlockRow, AppError> {
    let row = sqlx::query_as::<_, TangleBlockRow>(
        "INSERT INTO public.tangle_blocks (id, parents, data, signature, public_key) VALUES (COALESCE($1, gen_random_uuid()), $2, $3, $4, $5) RETURNING id, parents, data, signature, public_key, created_at"
    )
    .bind(id)
    .bind(parents)
    .bind(data)
    .bind(signature)
    .bind(public_key)
    .fetch_one(pool)
    .await
    .map_err(AppError::from)?;

    Ok(row)
}

pub async fn get_block(pool: &PgPool, id: Uuid) -> Result<TangleBlockRow, AppError> {
    let row = sqlx::query_as::<_, TangleBlockRow>(
        "SELECT id, parents, data, signature, public_key, created_at FROM public.tangle_blocks WHERE id = $1"
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(AppError::from)?;

    Ok(row)
}

pub async fn list_blocks(pool: &PgPool, per_page: i64, offset: i64) -> Result<(Vec<TangleBlockRow>, i64), AppError> {
    let rows: Vec<TangleBlockRow> = sqlx::query_as::<_, TangleBlockRow>(
        "SELECT id, parents, data, signature, public_key, created_at FROM public.tangle_blocks ORDER BY created_at DESC LIMIT $1 OFFSET $2"
    )
    .bind(per_page)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(AppError::from)?;

    let total: i64 = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM public.tangle_blocks")
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;

    Ok((rows, total))
}

pub async fn update_block(
    pool: &PgPool,
    id: Uuid,
    parents: Option<Vec<String>>,
    data: Option<Value>,
    signature: Option<Vec<u8>>,
    public_key: Option<String>,
) -> Result<TangleBlockRow, AppError> {
    let row = sqlx::query_as::<_, TangleBlockRow>(
        "UPDATE public.tangle_blocks SET parents = COALESCE($1, parents), data = COALESCE($2, data), signature = COALESCE($3, signature), public_key = COALESCE($4, public_key) WHERE id = $5 RETURNING id, parents, data, signature, public_key, created_at"
    )
    .bind(parents)
    .bind(data)
    .bind(signature)
    .bind(public_key)
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(AppError::from)?;

    Ok(row)
}

pub async fn delete_block(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
    sqlx::query("DELETE FROM public.tangle_blocks WHERE id = $1").bind(id).execute(pool).await.map_err(AppError::from)?;
    Ok(())
}
