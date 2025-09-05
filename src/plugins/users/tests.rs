#[cfg(test)]
mod tests {

    use crate::db;
    use crate::kernel::build_app;
    use crate::plugins::health::HealthPlugin;
    use axum::body::Body;
    use axum::http::{Method, Request, StatusCode};
    use serde::Deserialize;
    use serde_json::json;
    use sqlx::PgPool;
    use std::env;
    use tower::util::ServiceExt;

    #[derive(Deserialize)]
    struct RespUser {
        id: uuid::Uuid,
        username: String,
    }

    #[tokio::test]
    async fn users_crud_flow() -> anyhow::Result<()> {
        let test_db_url = env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost:5432/ecoblock_test".to_string()
        });

        let mut maintenance_url = test_db_url.clone();
        if let Some(idx) = maintenance_url.rfind('/') {
            maintenance_url.replace_range(idx + 1.., "postgres");
        }

        let db_name = test_db_url
            .rsplit('/')
            .next()
            .unwrap()
            .split('?')
            .next()
            .unwrap();

        let maint_pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect(&maintenance_url)
            .await?;

        let _ = sqlx::query(&format!("DROP DATABASE IF EXISTS \"{}\"", db_name))
            .execute(&maint_pool)
            .await;
        sqlx::query(&format!("CREATE DATABASE \"{}\"", db_name))
            .execute(&maint_pool)
            .await?;

        let test_pool_for_ext = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect(&test_db_url)
            .await?;
        let _ = sqlx::query("CREATE EXTENSION IF NOT EXISTS pgcrypto")
            .execute(&test_pool_for_ext)
            .await;

        let pool: PgPool = db::init_db(&test_db_url).await?;

        let users_plugin = crate::plugins::users::UsersPlugin::new(pool.clone());
        let plugins: Vec<Box<dyn crate::kernel::Plugin>> =
            vec![Box::new(HealthPlugin), Box::new(users_plugin)];
    let app = build_app(&plugins, None, None, None).await;

        let req = Request::builder()
            .method(Method::GET)
            .uri("/health")
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        eprintln!("health -> {}", resp.status());

        let payload = json!({
            "username": "testuser",
            "email": "test@example.com",
            "password": "password123"
        });

        let req = Request::builder()
            .method(Method::POST)
            .uri("/users")
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap();

        let resp = app.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let body_bytes = axum::body::to_bytes(resp.into_body(), 64 * 1024).await?;
        eprintln!(
            "create -> {} -> {}",
            status,
            String::from_utf8_lossy(&body_bytes)
        );
        assert!(
            status.is_success(),
            "create failed: {} -> {}",
            status,
            String::from_utf8_lossy(&body_bytes)
        );
        let created: RespUser = serde_json::from_slice(&body_bytes)?;

        let req = Request::builder()
            .method(Method::GET)
            .uri("/users")
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let body_bytes = axum::body::to_bytes(resp.into_body(), 64 * 1024).await?;
        eprintln!(
            "list -> {} -> {}",
            status,
            String::from_utf8_lossy(&body_bytes)
        );
        assert!(
            status.is_success(),
            "list failed: {} -> {}",
            status,
            String::from_utf8_lossy(&body_bytes)
        );

        let req = Request::builder()
            .method(Method::GET)
            .uri(format!("/users/{}", created.id))
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let body_bytes = axum::body::to_bytes(resp.into_body(), 64 * 1024).await?;
        eprintln!(
            "get -> {} -> {}",
            status,
            String::from_utf8_lossy(&body_bytes)
        );
        assert!(
            status.is_success(),
            "get failed: {} -> {}",
            status,
            String::from_utf8_lossy(&body_bytes)
        );
        let got: RespUser = serde_json::from_slice(&body_bytes)?;
        assert_eq!(got.id, created.id);

        let update = json!({ "username": "updated", "email": "new@example.com" });
        let req = Request::builder()
            .method(Method::PUT)
            .uri(format!("/users/{}", created.id))
            .header("content-type", "application/json")
            .body(Body::from(update.to_string()))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let body_bytes = axum::body::to_bytes(resp.into_body(), 64 * 1024).await?;
        eprintln!(
            "update -> {} -> {}",
            status,
            String::from_utf8_lossy(&body_bytes)
        );
        assert!(
            status.is_success(),
            "update failed: {} -> {}",
            status,
            String::from_utf8_lossy(&body_bytes)
        );
        let updated: RespUser = serde_json::from_slice(&body_bytes)?;
        assert_eq!(updated.username, "updated");

        let req = Request::builder()
            .method(Method::DELETE)
            .uri(format!("/users/{}", created.id))
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let body_bytes = axum::body::to_bytes(resp.into_body(), 64 * 1024).await?;
        eprintln!(
            "delete -> {} -> {}",
            status,
            String::from_utf8_lossy(&body_bytes)
        );
        assert_eq!(status, StatusCode::NO_CONTENT);

        let req = Request::builder()
            .method(Method::GET)
            .uri(format!("/users/{}", created.id))
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        Ok(())
    }

        #[test]
        fn create_user_model_deserialize_and_fields() {
            use crate::plugins::users::models::CreateUser;
            let json = r#"{"username":"testuser","email":"test@example.com","password":"secretpass"}"#;
            let cu: CreateUser = serde_json::from_str(json).expect("should deserialize CreateUser");
            assert_eq!(cu.username, "testuser");
            assert_eq!(cu.email, "test@example.com");
            assert_eq!(cu.password, "secretpass");
        }

        #[test]
        fn update_user_model_optional_fields() {
            use crate::plugins::users::models::UpdateUser;
            let json = r#"{}"#;
            let u: UpdateUser = serde_json::from_str(json).expect("should deserialize UpdateUser");
            assert!(u.username.is_none());
            assert!(u.email.is_none());

            let json2 = r#"{"username":"newname"}"#;
            let u2: UpdateUser = serde_json::from_str(json2).expect("should deserialize UpdateUser partial");
            assert_eq!(u2.username.unwrap(), "newname");
            assert!(u2.email.is_none());
        }

        #[test]
        fn user_dto_serializes_with_expected_fields() {
            use crate::plugins::users::models::UserDto;
            use uuid::Uuid;
            let id = Uuid::new_v4();
            let dto = UserDto {
                id,
                username: "u1".to_string(),
                email: "e@x.com".to_string(),
            };
            let s = serde_json::to_string(&dto).expect("should serialize UserDto");
            assert!(s.contains(&id.to_string()));
            assert!(s.contains("u1"));
            assert!(s.contains("e@x.com"));
        }
}
