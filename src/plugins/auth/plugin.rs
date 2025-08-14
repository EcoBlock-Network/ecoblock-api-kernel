use axum::{Router, routing::{post, get}, middleware};
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
        let pool1 = self.pool.clone();
        let pool2 = self.pool.clone();
        let public = Router::new()
            .route("/login", post(move |payload| handlers::login(axum::extract::State(pool1.clone()), payload)));

        let protected = Router::new()
            .route("/whoami", get(move |auth| handlers::whoami(axum::extract::State(pool2.clone()), auth)))
            .layer(middleware::from_fn(crate::plugins::auth::middleware::require_auth));

        public.merge(protected)
    }

    fn name(&self) -> &'static str { "auth" }
}
