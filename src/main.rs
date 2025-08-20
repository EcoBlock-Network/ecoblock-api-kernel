mod kernel;
mod plugins;
mod http_error;

use axum::Router;
// ...existing code... (middleware Next is used in kernel layer)
use kernel::{build_app, Plugin};
use plugins::health::HealthPlugin;
use plugins::auth::AuthPlugin;
use crate::plugins::communication::blog::plugin::BlogPlugin;
use crate::plugins::communication::stories::plugin::StoriesPlugin;
use crate::plugins::metrics::MetricsPlugin;
use crate::plugins::tangle::plugin::TanglePlugin;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use dotenvy::dotenv;
use std::env;
// CORS handled in kernel::build_app for local dev
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
    let tangle_plugin = TanglePlugin::new(_pool.clone());
    let plugins_vec: Vec<Box<dyn Plugin>> = vec![
        Box::new(HealthPlugin),
        Box::new(users_plugin),
        Box::new(auth_plugin),
        Box::new(tangle_plugin),
        Box::new(blog_plugin),
        Box::new(stories_plugin),
    ];

    let plugin_names: Vec<&'static str> = plugins_vec.iter().map(|p| p.name()).collect();
    tracing::info!("mounting plugins: {:?}", plugin_names);

    // build app and pass metrics plugin so each plugin router can be instrumented with route labels
    let mut app: Router = build_app(&plugins_vec, Some(metrics_plugin.clone())).await;

    // expose metrics at /metrics (not instrumented to avoid double-counting)
    app = app.nest("/metrics", metrics_plugin.router());

    for p in plugins_vec.iter() {
        tracing::info!("mounted plugin: {}", p.name());
    }

    let port: u16 = env::var("PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(3000);
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse()?;
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("listening on {}", addr);

    // In dev, optionally spawn the web-admin dev server so it's automatically available.
    if std::env::var("START_WEB_ADMIN").map(|s| s == "true").unwrap_or(false) {
        let api_base = format!("http://127.0.0.1:{}", port);
        let dev_token = std::env::var("VITE_DEV_TOKEN").ok();
        tokio::spawn(async move {
            use std::process::Command;
            let mut cmd = Command::new("npm");
            cmd.arg("run").arg("dev").current_dir("web-admin");
            cmd.env("VITE_API_BASE", &api_base);
            if let Some(t) = dev_token {
                cmd.env("VITE_DEV_TOKEN", t);
            }
            // best-effort spawn; ignore failures
            let _ = cmd.spawn();
        });
    }

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
