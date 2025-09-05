use std::time::Duration;

mod common;

#[tokio::test]
async fn redis_cache_get_set_delete_smoke() -> anyhow::Result<()> {
    let redis_url = match std::env::var("REDIS_URL") {
        Ok(u) => u,
        Err(_) => {
            eprintln!("SKIPPING redis_cache_get_set_delete_smoke: REDIS_URL not set");
            return Ok(());
        }
    };

    let cache = match ecoblock_api_kernel::cache::RedisCache::new(&redis_url).await {
        Ok(c) => c.into_arc(),
        Err(e) => {
            eprintln!(
                "SKIPPING redis_cache_get_set_delete_smoke: cannot connect to Redis: {}",
                e
            );
            return Ok(());
        }
    };

    let key = "test:redis:smoke";
    cache
        .set(key, b"hello".to_vec(), Some(Duration::from_secs(5)))
        .await?;
    let got = cache.get(key).await?;
    assert_eq!(got, Some(b"hello".to_vec()));
    cache.delete(key).await?;
    let got2 = cache.get(key).await?;
    assert_eq!(got2, None);
    Ok(())
}
