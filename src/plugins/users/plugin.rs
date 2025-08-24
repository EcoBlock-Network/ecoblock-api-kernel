use crate::cache::DynCache;
use crate::kernel::Plugin;
use crate::plugins::users::handlers::{
    create_admin, create_user, delete_user, get_user, grant_admin, list_users, update_user,
};
use crate::plugins::users::models::{CreateUser, UpdateUser};
use axum::{
    Json, Router,
    extract::Extension,
    extract::Path,
    routing::{delete, get, post, put},
};
use sqlx::PgPool;

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
        let p_grant = self.pool.clone();
        let p_admin = self.pool.clone();

        Router::new()
            .route(
                "/",
                post(move |Json(payload): Json<CreateUser>| {
                    let pool = p_create.clone();
                    async move { create_user(pool, payload).await }
                }),
            )
            .route(
                "/",
                get(move || {
                    let pool = p_list.clone();
                    async move { list_users(pool).await }
                }),
            )
            .route(
                "/:id",
                get(
                    move |ext: Extension<Option<DynCache>>, Path(id): Path<uuid::Uuid>| {
                        let pool = p_get.clone();
                        async move { get_user(ext, pool, Path(id)).await }
                    },
                ),
            )
            .route(
                "/:id",
                put(
                    move |ext: Extension<Option<DynCache>>,
                          Path(id): Path<uuid::Uuid>,
                          Json(payload): Json<UpdateUser>| {
                        let pool = p_update.clone();
                        async move { update_user(ext, pool, Path(id), Json(payload)).await }
                    },
                ),
            )
            .route(
                "/:id",
                delete(
                    move |ext: Extension<Option<DynCache>>, Path(id): Path<uuid::Uuid>| {
                        let pool = p_delete.clone();
                        async move { delete_user(ext, pool, Path(id)).await }
                    },
                ),
            )
            .route(
                "/:id/grant_admin",
                post(
                    move |ext: Extension<Option<DynCache>>,
                          Path(id): Path<uuid::Uuid>,
                          auth: crate::plugins::auth::handlers::AuthUser| {
                        let pool = p_grant.clone();
                        async move { grant_admin(ext, pool, auth, Path(id)).await }
                    },
                )
                .layer(axum::middleware::from_fn(
                    crate::plugins::auth::middleware::require_auth,
                )),
            )
            .route(
                "/admin",
                post(
                    move |ext: Extension<Option<DynCache>>,
                          auth: crate::plugins::auth::handlers::AuthUser,
                          Json(payload): Json<CreateUser>| {
                        let pool = p_admin.clone();
                        async move { create_admin(ext, pool, auth, payload).await }
                    },
                )
                .layer(axum::middleware::from_fn(
                    crate::plugins::auth::middleware::require_auth,
                )),
            )
    }

    fn name(&self) -> &'static str {
        "users"
    }
}
