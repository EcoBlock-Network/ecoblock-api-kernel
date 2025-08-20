use crate::plugins::tangle::repo;
use crate::plugins::tangle::models::TangleBlockCreate;
use sqlx::PgPool;
use serde_json::json;
use std::env;

#[tokio::test]
async fn repo_crud_cycle() {
    let database_url = env::var("TEST_DATABASE_URL").unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/ecoblock_test".to_string());
    let pool = PgPool::connect(&database_url).await.expect("connect db");

    // simple create
    let create = TangleBlockCreate {
        id: None,
        parents: vec!["parent1".to_string()],
        data: json!({"foo":"bar"}),
        signature: base64::engine::general_purpose::STANDARD.encode(b"sig"),
        public_key: "pk1".to_string(),
    };

    let inserted = repo::insert_block(&pool, create.id, &create.parents, &create.data, b"sig", &create.public_key).await.expect("insert");
    let fetched = repo::get_block(&pool, inserted.id).await.expect("get");
    assert_eq!(fetched.public_key, "pk1");

    let (rows, total) = repo::list_blocks(&pool, 10, 0).await.expect("list");
    assert!(total >= 1);

    let updated = repo::update_block(&pool, inserted.id, Some(vec!["p2".to_string()]), None, None, None).await.expect("update");
    assert_eq!(updated.parents, vec!["p2".to_string()]);

    repo::delete_block(&pool, inserted.id).await.expect("delete");
}
