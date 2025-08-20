use sqlx::PgPool;
use crate::http_error::AppError;
use uuid::Uuid;
use sqlx::Row;

pub async fn find_user_by_username(pool: &PgPool, username: &str) -> Result<Option<(Uuid, String)>, AppError> {
    let opt = sqlx::query("SELECT id, password_hash FROM users WHERE username = $1")
        .bind(username)
        .fetch_optional(pool)
        .await
        .map_err(AppError::from)?;

    if let Some(r) = opt {
        let id: Uuid = r.get("id");
        let password_hash: String = r.get("password_hash");
        Ok(Some((id, password_hash)))
    } else {
        Ok(None)
    }
}

pub async fn get_user_basic(pool: &PgPool, id: Uuid) -> Result<(Uuid, String, String), AppError> {
    let r = sqlx::query("SELECT id, username, email FROM users WHERE id = $1")
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;
    Ok((r.get("id"), r.get("username"), r.get("email")))
}
