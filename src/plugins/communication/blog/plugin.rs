use axum::{Router, routing::get, routing::post, routing::put, routing::delete, Extension};
use crate::kernel::Plugin;
use crate::plugins::communication::blog::handlers::*;
use sqlx::PgPool;
use std::sync::Arc;

pub struct BlogPlugin { pub pool: Arc<PgPool> }

impl BlogPlugin {
    pub fn new(pool: PgPool) -> Self { Self { pool: Arc::new(pool) } }
}

#[async_trait::async_trait]
impl Plugin for BlogPlugin {
    async fn router(&self) -> Router {
        Router::new()
            .route("/", post(create_blog))
            .route("/", get(list_blogs))
            .route("/:id", get(get_blog))
            .route("/:id", put(update_blog))
            .route("/:id", delete(delete_blog))
            .layer(Extension(self.pool.clone()))
    }

    fn name(&self) -> &'static str { "communication/blog" }
}
