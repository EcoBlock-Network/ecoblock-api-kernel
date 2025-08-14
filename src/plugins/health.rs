use axum::{Router, routing::get, Json};
use serde::Serialize;
use crate::kernel::Plugin;

#[derive(Serialize)]
struct Health {
    status: &'static str,
}

pub struct HealthPlugin;

#[axum::debug_handler]
async fn health_handler() -> Json<Health> {
    Json(Health { status: "ok" })
}

#[async_trait::async_trait]
impl Plugin for HealthPlugin {
    async fn router(&self) -> Router {
        // expose health at root of the plugin mount point
        Router::new().route("/", get(health_handler))
    }

    fn name(&self) -> &'static str {
        "health"
    }

    async fn on_start(&self) {
        tracing::info!("health plugin started");
    }
}
