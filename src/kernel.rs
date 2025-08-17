use axum::Router;
use async_trait::async_trait;
use tracing::info;


#[async_trait]
pub trait Plugin: Send + Sync {
    
    async fn router(&self) -> Router;
    
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
