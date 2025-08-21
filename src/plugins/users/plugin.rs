use axum::{Router, routing::{post, get, put, delete}, Json, extract::Path};
use sqlx::PgPool;
use crate::kernel::Plugin;
use crate::plugins::users::models::{CreateUser, UpdateUser};
use crate::plugins::users::handlers::{create_user, list_users, get_user, update_user, delete_user};

pub struct UsersPlugin {
    pub pool: PgPool,
}

impl UsersPlugin {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Plugin for UsersPlugin {
    async fn router(&self) -> Router {
        
        let p_create = self.pool.clone();
        let p_list = self.pool.clone();
        let p_get = self.pool.clone();
        let p_update = self.pool.clone();
    let p_delete = self.pool.clone();
        let p_grant = self.pool.clone(); // Cloning the pool for grant_admin endpoint
        let p_admin = self.pool.clone(); // separate clone for the /admin route to avoid move issues

        Router::new()
            .route("/", post(move |Json(payload): Json<CreateUser>| {
                let pool = p_create.clone();
                async move { create_user(pool, payload).await }
            }))
            .route("/", get(move || {
                let pool = p_list.clone();
                async move { list_users(pool).await }
            }))
            .route("/:id", get(move |Path(id): Path<uuid::Uuid>| {
                let pool = p_get.clone();
                async move { get_user(pool, Path(id)).await }
            }))
            .route("/:id", put(move |Path(id): Path<uuid::Uuid>, Json(payload): Json<UpdateUser>| {
                let pool = p_update.clone();
                async move { update_user(pool, Path(id), Json(payload)).await }
            }))
            .route("/:id", delete(move |Path(id): Path<uuid::Uuid>| {
                let pool = p_delete.clone();
                async move { delete_user(pool, Path(id)).await }
            }))
            .route("/:id/grant_admin", post(move |Path(id): Path<uuid::Uuid>, auth: crate::plugins::auth::handlers::AuthUser| {
                let pool = p_grant.clone();
                async move { crate::plugins::users::handlers::grant_admin(pool, auth, Path(id)).await }
            }).layer(axum::middleware::from_fn(crate::plugins::auth::middleware::require_auth)))
                    .route("/admin", post(move |auth: crate::plugins::auth::handlers::AuthUser, Json(payload): Json<CreateUser>| {
                        let pool = p_admin.clone();
                        async move { crate::plugins::users::handlers::create_admin(pool, auth, payload).await }
                    }).layer(axum::middleware::from_fn(crate::plugins::auth::middleware::require_auth)))
    }

    fn name(&self) -> &'static str {
        "users"
    }
}
