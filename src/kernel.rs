use axum::Router;
use axum::middleware::Next;
use axum::body::Body;
use crate::plugins::metrics::MetricsPlugin as MaybeMetrics;
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
pub async fn build_app(plugins: &Vec<Box<dyn Plugin>>, metrics: Option<MaybeMetrics>) -> Router {
    let mut app = Router::new();

    for plugin in plugins.iter() {
        info!("starting plugin {}", plugin.name());
        plugin.on_start().await;
        let mut router = plugin.router().await;

        // if metrics plugin provided, wrap the plugin router with a middleware
        // that records request duration and counts using the matched route
        if let Some(ref m) = metrics {
            let counter = m.request_counter.clone();
            let histogram = m.request_duration.clone();
            router = router.layer(axum::middleware::from_fn(move |req: axum::http::Request<Body>, next: Next| {
                let counter = counter.clone();
                let histogram = histogram.clone();
                async move {
                    let method = req.method().to_string();
                    // MatchedPath should be populated for routers inside this layer
                    let route_label = if let Some(matched) = req.extensions().get::<axum::extract::MatchedPath>() {
                        matched.as_str().to_string()
                    } else {
                        req.uri().path().to_string()
                    };
                    let start = std::time::Instant::now();
                    let res = next.run(req).await;
                    let elapsed = start.elapsed();
                    let secs = elapsed.as_secs_f64();
                    histogram.with_label_values(&[&method, &route_label]).observe(secs);
                    let status = res.status().as_u16().to_string();
                    counter.with_label_values(&[&method, &route_label, &status]).inc();
                    res
                }
            }));
        }

        // mount plugin under its name to namespace routes
        app = app.nest(&format!("/{}", plugin.name()), router);
    }

    app
}
