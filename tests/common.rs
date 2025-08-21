use std::process::Command;
use tokio::net::TcpListener;
use std::sync::Once;
use ecoblock_api_kernel::db;
use ecoblock_api_kernel::kernel::build_app;

static JWT_INIT: Once = Once::new();
const JWT_SECRET_CONST: &str = "ecoblock-test-secret";

pub struct TestDbGuard {
    maintenance_url: String,
    unique_db: String,
}

impl TestDbGuard {
    pub fn new(maintenance_url: String, unique_db: String) -> Self {
        Self { maintenance_url, unique_db }
    }
}

impl Drop for TestDbGuard {
    fn drop(&mut self) {
        let _ = Command::new("psql")
            .arg(&self.maintenance_url)
            .arg("-c")
            .arg(format!(
                "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{}' AND pid <> pg_backend_pid();",
                self.unique_db
            ))
            .status();
        let _ = Command::new("psql")
            .arg(&self.maintenance_url)
            .arg("-c")
            .arg(format!("DROP DATABASE IF EXISTS \"{}\"", self.unique_db))
            .status();
    }
}

pub async fn setup_and_spawn(test_db: &str) -> anyhow::Result<(String, tokio::task::JoinHandle<()>, String, sqlx::PgPool, TestDbGuard)> {
    // backward-compatible convenience: build users+auth app
    let (pool, guard) = create_test_db_and_pool(test_db).await?;
    let users_plugin = ecoblock_api_kernel::plugins::users::UsersPlugin::new(pool.clone());
    let auth_plugin = ecoblock_api_kernel::plugins::auth::AuthPlugin::new(pool.clone());
    let plugins: Vec<Box<dyn ecoblock_api_kernel::kernel::Plugin>> = vec![Box::new(ecoblock_api_kernel::plugins::health::HealthPlugin), Box::new(users_plugin), Box::new(auth_plugin)];
    let (base, server_handle) = spawn_app_with_plugins(pool.clone(), plugins).await?;
    let jwt_secret = JWT_SECRET_CONST.to_string();
    Ok((base, server_handle, jwt_secret, pool, guard))
}

pub async fn create_test_db_and_pool(test_db: &str) -> anyhow::Result<(sqlx::PgPool, TestDbGuard)> {
    let maintenance = test_db.to_string();
    let mut maintenance_url = maintenance.clone();
    if let Some(idx) = maintenance_url.rfind('/') {
        maintenance_url.replace_range(idx + 1.., "postgres");
    }
    let base_db_name = test_db.rsplit('/').next().unwrap().split('?').next().unwrap();
    let unique_db = format!("{}_{}", base_db_name, uuid::Uuid::new_v4().to_string().replace('-', ""));
    let mut unique_db_url = test_db.to_string();
    if let Some(idx) = unique_db_url.rfind('/') {
        unique_db_url.replace_range(idx + 1.., &unique_db);
    }
    let _ = Command::new("psql").arg(&maintenance_url).arg("-c").arg(format!("DROP DATABASE IF EXISTS \"{}\"", unique_db)).status();
    let _ = Command::new("psql").arg(&maintenance_url).arg("-c").arg(format!("CREATE DATABASE \"{}\"", unique_db)).status();
    let _ = Command::new("psql").arg(&unique_db_url).arg("-c").arg("CREATE EXTENSION IF NOT EXISTS pgcrypto;").status();
    let guard = TestDbGuard::new(maintenance_url.clone(), unique_db.clone());
    JWT_INIT.call_once(|| {
        unsafe { std::env::set_var("JWT_SECRET", JWT_SECRET_CONST); }
    });
    let pool = db::init_db(&unique_db_url).await?;
    Ok((pool, guard))
}

pub async fn spawn_app_with_plugins(pool: sqlx::PgPool, plugins: Vec<Box<dyn ecoblock_api_kernel::kernel::Plugin>>) -> anyhow::Result<(String, tokio::task::JoinHandle<()>)> {
    let app = build_app(&plugins, None).await;
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let server_handle = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("server error");
    });
    Ok((format!("http://{}", addr), server_handle))
}
