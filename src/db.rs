use sqlx::{Pool, Postgres};

pub type DbPool = Pool<Postgres>;

pub async fn init_db(database_url: &str) -> anyhow::Result<DbPool> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    // Run migrations embedded at compile time (migrations/)
    sqlx::migrate!().run(&pool).await?;

    Ok(pool)
}
