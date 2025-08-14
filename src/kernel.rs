use axum::Router;
use async_trait::async_trait;
use tracing::info;

/// A plugin provides a router to be mounted into the main application router.
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Returns a router for this plugin (routes should be relative to root).
    async fn router(&self) -> Router;
    /// Short name used for mounting and logs (should be url-safe).
    fn name(&self) -> &'static str;
    /// Optional lifecycle hook called when the kernel starts.
    async fn on_start(&self) {}
    /// Optional lifecycle hook called on shutdown.
    async fn on_shutdown(&self) {}
}

/// Builds the application router by mounting each plugin under `/{plugin.name()}`.
pub async fn build_app(plugins: &Vec<Box<dyn Plugin>>) -> Router {
    let mut app = Router::new();

    for plugin in plugins.iter() {
        info!("starting plugin {}", plugin.name());
        plugin.on_start().await;
        let router = plugin.router().await;
        // mount plugin under its name to namespace routes
        app = app.nest(&format!("/{}", plugin.name()), router);
    }

    app
}
