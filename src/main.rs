mod kernel;
mod plugins;

use axum::Router;
use kernel::{build_app, Plugin};
use plugins::health::HealthPlugin;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use dotenvy::dotenv;
use std::env;

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
    let plugins_vec: Vec<Box<dyn Plugin>> = vec![Box::new(HealthPlugin), Box::new(users_plugin)];

    let app: Router = build_app(&plugins_vec).await;

    let addr: SocketAddr = "0.0.0.0:3000".parse()?;
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
