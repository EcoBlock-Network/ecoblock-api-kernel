use axum::{Router, routing::post};
use crate::kernel::Plugin;
use async_trait::async_trait;
use sqlx::PgPool;
use crate::plugins::auth::handlers;

pub struct AuthPlugin {
    pool: PgPool,
}

impl AuthPlugin {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
}

#[async_trait]
impl Plugin for AuthPlugin {
    async fn router(&self) -> Router {
        Router::new().route("/login", post({
            let pool = self.pool.clone();
            move |payload| handlers::login(axum::extract::State(pool.clone()), payload)
        }))
    }

    fn name(&self) -> &'static str { "auth" }
}
