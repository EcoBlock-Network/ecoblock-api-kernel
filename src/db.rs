use sqlx::{Pool, Postgres, Error as SqlxError};

pub type DbPool = Pool<Postgres>;

fn sanitize_sqlx_error(e: &SqlxError) -> String {
    use sqlx::Error::*;
    match e {
        Database(db) => {
            let code = db.code().map(|c| c.to_string()).unwrap_or_else(|| "<unknown>".to_string());
            let cons = db.constraint().map(|c| c.to_string());
            match cons {
                Some(c) => format!("database error (sqlstate={}, constraint={})", code, c),
                None => format!("database error (sqlstate={})", code),
            }
        }
        other => {
            format!("sqlx error: {:?}", other)
        }
    }
}

pub async fn init_db(database_url: &str) -> anyhow::Result<DbPool> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
        .map_err(|e| anyhow::anyhow!(sanitize_sqlx_error(&e)))?;

    sqlx::migrate!()
        .run(&pool)
        .await
        .map_err(|e| anyhow::anyhow!(format!("migration error: {:?}", e)))?;

    Ok(pool)
}
