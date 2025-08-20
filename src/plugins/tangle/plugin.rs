use axum::{Router, routing::get, routing::post, routing::put, routing::delete, Extension};
use crate::kernel::Plugin;
use crate::plugins::tangle::handlers::*;
use sqlx::PgPool;

pub struct TanglePlugin { pub pool: PgPool }

impl TanglePlugin {
    #[allow(dead_code)]
    pub fn new(pool: PgPool) -> Self { Self { pool } }
}

#[async_trait::async_trait]
impl Plugin for TanglePlugin {
    async fn router(&self) -> Router {
        Router::new()
            .route("/", post(create_block))
            .route("/", get(list_blocks))
            .route("/:id", get(get_block))
            .route("/:id", put(update_block))
            .route("/:id", delete(delete_block))
            .layer(Extension(self.pool.clone()))
    }

    fn name(&self) -> &'static str { "tangle" }
}
