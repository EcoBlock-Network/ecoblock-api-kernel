use crate::cache::DynCache;
use crate::plugins::metrics::MetricsPlugin as MaybeMetrics;
use async_trait::async_trait;
use axum::Router;
use axum::body::Body;
use axum::http::{
    Method, Request, Response, StatusCode,
    header::{HeaderValue, ORIGIN},
};
use axum::middleware::Next;
use tracing::info;

#[async_trait]
pub trait Plugin: Send + Sync {
    async fn router(&self) -> Router;
    fn name(&self) -> &'static str;
    async fn on_start(&self) {}
    async fn on_shutdown(&self) {}
}

pub async fn build_app(
    plugins: &Vec<Box<dyn Plugin>>,
    metrics: Option<MaybeMetrics>,
    cache: Option<DynCache>,
) -> Router {
    let mut app = Router::new();

    for plugin in plugins.iter() {
        info!("starting plugin {}", plugin.name());
        plugin.on_start().await;
        let mut router = plugin.router().await;

        if let Some(ref m) = metrics {
            let counter = m.request_counter.clone();
            let histogram = m.request_duration.clone();
            router = router.layer(axum::middleware::from_fn(
                move |req: axum::http::Request<Body>, next: Next| {
                    let counter = counter.clone();
                    let histogram = histogram.clone();
                    async move {
                        let method = req.method().to_string();
                        let route_label = if let Some(matched) =
                            req.extensions().get::<axum::extract::MatchedPath>()
                        {
                            matched.as_str().to_string()
                        } else {
                            req.uri().path().to_string()
                        };
                        let start = std::time::Instant::now();
                        let res = next.run(req).await;
                        let elapsed = start.elapsed();
                        let secs = elapsed.as_secs_f64();
                        histogram
                            .with_label_values(&[&method, &route_label])
                            .observe(secs);
                        let status = res.status().as_u16().to_string();
                        counter
                            .with_label_values(&[&method, &route_label, &status])
                            .inc();
                        res
                    }
                },
            ));
        }

        let router = router.layer(axum::middleware::from_fn(
            |req: Request<Body>, next: Next| async move {
                let allowed_env = std::env::var("ALLOWED_ORIGINS").ok();
                let allowed_list: Vec<String> = allowed_env
                    .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
                    .unwrap_or_else(|| {
                        vec![
                            "http://localhost:5173".to_string(),
                            "http://localhost:5174".to_string(),
                        ]
                    });

                let origin_hdr = req
                    .headers()
                    .get(ORIGIN)
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.to_string());
                let allowed_origin = origin_hdr
                    .as_deref()
                    .filter(|o| allowed_list.iter().any(|a| a == o));

                if req.method() == Method::OPTIONS {
                    let mut res = Response::new(Body::empty());
                    *res.status_mut() = StatusCode::OK;
                    if let Some(o) = allowed_origin {
                        if let Ok(hv) = HeaderValue::from_str(o) {
                            res.headers_mut().insert("access-control-allow-origin", hv);
                        }
                    }
                    res.headers_mut().insert(
                        "access-control-allow-methods",
                        HeaderValue::from_static("GET,POST,PUT,DELETE,OPTIONS"),
                    );
                    res.headers_mut().insert(
                        "access-control-allow-headers",
                        HeaderValue::from_static("*"),
                    );
                    return res;
                }

                let mut res = next.run(req).await;
                if let Some(o) = allowed_origin {
                    if let Ok(hv) = HeaderValue::from_str(o) {
                        res.headers_mut().insert("access-control-allow-origin", hv);
                    }
                }
                res.headers_mut().insert(
                    "access-control-allow-headers",
                    HeaderValue::from_static("*"),
                );
                res
            },
        ));

        let ext_cache = cache.clone();
        app = app.nest(
            &format!("/{}", plugin.name()),
            router.layer(axum::Extension::<Option<DynCache>>(ext_cache)),
        );
    }

    app
}
