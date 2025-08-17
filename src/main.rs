mod kernel;
mod plugins;
mod http_error;

use axum::Router;
use axum::middleware::Next;
use kernel::{build_app, Plugin};
use plugins::health::HealthPlugin;
use plugins::auth::AuthPlugin;
use crate::plugins::communication::blog::plugin::BlogPlugin;
use crate::plugins::communication::stories::plugin::StoriesPlugin;
use crate::plugins::metrics::MetricsPlugin;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use dotenvy::dotenv;
use std::env;
// tower imports intentionally omitted

mod db;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    // load environment and initialize DB
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/ecoblock".to_string());
    let _pool = db::init_db(&database_url).await?;

    // instantiate plugins
    let users_plugin = plugins::users::UsersPlugin::new(_pool.clone());
    let auth_plugin = AuthPlugin::new(_pool.clone());
    let blog_plugin = BlogPlugin::new(_pool.clone());
    let stories_plugin = StoriesPlugin::new(_pool.clone());
    let metrics_plugin = MetricsPlugin::new();
    let plugins_vec: Vec<Box<dyn Plugin>> = vec![
        Box::new(HealthPlugin),
        Box::new(users_plugin),
        Box::new(auth_plugin),
        Box::new(blog_plugin),
        Box::new(stories_plugin),
    ];

    let plugin_names: Vec<&'static str> = plugins_vec.iter().map(|p| p.name()).collect();
    tracing::info!("mounting plugins: {:?}", plugin_names);

    let mut app: Router = build_app(&plugins_vec).await;
    // mount metrics router at /metrics
    app = app.nest("/metrics", metrics_plugin.router());

    // add a middleware to increment Prometheus request counter and record duration
    let counter = metrics_plugin.request_counter.clone();
    let histogram = metrics_plugin.request_duration.clone();
    app = app.layer(axum::middleware::from_fn(move |req: axum::http::Request<axum::body::Body>, next: Next| {
        let counter = counter.clone();
        let histogram = histogram.clone();
        async move {
            let method = req.method().to_string();
            let path = req.uri().path().to_string();
            let start = std::time::Instant::now();
            let res = next.run(req).await;
            let elapsed = start.elapsed();
            let secs = elapsed.as_secs_f64();
            // observe duration (labels: method, path)
            histogram.with_label_values(&[&method, &path]).observe(secs);
            let status = res.status().as_u16().to_string();
            counter.with_label_values(&[&method, &path, &status]).inc();
            res
        }
    }));

    for p in plugins_vec.iter() {
        tracing::info!("mounted plugin: {}", p.name());
    }

    let port: u16 = env::var("PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(3000);
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse()?;
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("listening on {}", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            let _ = tokio::signal::ctrl_c().await;
            // call plugin shutdown hooks
            for p in plugins_vec.iter() {
                p.on_shutdown().await;
            }
        })
        .await?;

    Ok(())
}
