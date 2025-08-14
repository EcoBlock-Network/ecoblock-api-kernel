use axum::{Router, routing::get, routing::post, routing::put, routing::delete, Extension};
use crate::kernel::Plugin;
use crate::plugins::communication::stories::handlers::*;
use sqlx::PgPool;
use std::sync::Arc;

pub struct StoriesPlugin { pub pool: Arc<PgPool> }

impl StoriesPlugin {
    pub fn new(pool: PgPool) -> Self { Self { pool: Arc::new(pool) } }
}

#[async_trait::async_trait]
impl Plugin for StoriesPlugin {
    async fn router(&self) -> Router {
        Router::new()
            .route("/", post(create_story))
            .route("/", get(list_stories))
            .route("/:id", get(get_story))
            .route("/:id", put(update_story))
            .route("/:id", delete(delete_story))
            .layer(Extension(self.pool.clone()))
    }

    fn name(&self) -> &'static str { "communication/stories" }
}
