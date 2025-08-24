use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;

#[async_trait]
pub trait Cache: Send + Sync + 'static {
    async fn get(&self, key: &str) -> anyhow::Result<Option<Vec<u8>>>;
    async fn set(&self, key: &str, value: Vec<u8>, ttl: Option<Duration>) -> anyhow::Result<()>;
    async fn delete(&self, key: &str) -> anyhow::Result<()>;
}

pub type DynCache = Arc<dyn Cache>;

mod inmem {
    use super::*;
    use lru::LruCache;
    use parking_lot::Mutex;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    pub struct InMemoryCache {
        inner: Mutex<LruCache<u64, Vec<u8>>>,
    }

    impl InMemoryCache {
        pub fn new(capacity: usize) -> Self {
            use std::num::NonZeroUsize;
            let nz = NonZeroUsize::new(capacity).unwrap_or(NonZeroUsize::new(1).unwrap());
            Self {
                inner: Mutex::new(LruCache::new(nz)),
            }
        }

        fn hash_key(key: &str) -> u64 {
            let mut h = DefaultHasher::new();
            key.hash(&mut h);
            h.finish()
        }
    }

    #[async_trait]
    impl Cache for InMemoryCache {
        async fn get(&self, key: &str) -> anyhow::Result<Option<Vec<u8>>> {
            let k = Self::hash_key(key);
            Ok(self.inner.lock().get(&k).cloned())
        }

        async fn set(
            &self,
            key: &str,
            value: Vec<u8>,
            _ttl: Option<Duration>,
        ) -> anyhow::Result<()> {
            let k = Self::hash_key(key);
            self.inner.lock().put(k, value);
            Ok(())
        }

        async fn delete(&self, key: &str) -> anyhow::Result<()> {
            let k = Self::hash_key(key);
            self.inner.lock().pop(&k);
            Ok(())
        }
    }

    impl InMemoryCache {
        pub fn into_arc(self) -> DynCache {
            Arc::new(self)
        }
    }
}

pub use inmem::InMemoryCache;

mod redis_backend {
    use super::*;
    use redis::AsyncCommands;
    use redis::Client;
    use redis::aio::MultiplexedConnection;
    use std::sync::Arc as StdArc;
    use tokio::sync::Mutex as AsyncMutex;

    pub struct RedisCache {
        conn: StdArc<AsyncMutex<MultiplexedConnection>>,
    }

    impl RedisCache {
        pub async fn new(url: &str) -> anyhow::Result<Self> {
            let client = Client::open(url)?;
            let conn = client.get_multiplexed_tokio_connection().await?;
            Ok(Self {
                conn: StdArc::new(AsyncMutex::new(conn)),
            })
        }

        fn ttl_to_redis_seconds(ttl: Option<std::time::Duration>) -> Option<usize> {
            ttl.map(|d| d.as_secs() as usize)
        }
    }

    #[async_trait]
    impl Cache for RedisCache {
        async fn get(&self, key: &str) -> anyhow::Result<Option<Vec<u8>>> {
            let mut guard = self.conn.lock().await;
            let res: Option<Vec<u8>> = guard.get(key).await?;
            Ok(res)
        }

        async fn set(
            &self,
            key: &str,
            value: Vec<u8>,
            ttl: Option<std::time::Duration>,
        ) -> anyhow::Result<()> {
            let mut guard = self.conn.lock().await;
            if let Some(secs) = Self::ttl_to_redis_seconds(ttl) {
                let secs_u64: u64 = secs as u64;
                let _: () = redis::cmd("SETEX")
                    .arg(key)
                    .arg(secs_u64)
                    .arg(value)
                    .query_async(&mut *guard)
                    .await?;
            } else {
                let _: () = guard.set(key, value).await?;
            }
            Ok(())
        }

        async fn delete(&self, key: &str) -> anyhow::Result<()> {
            let mut guard = self.conn.lock().await;
            let _: () = guard.del(key).await?;
            Ok(())
        }
    }

    impl RedisCache {
        pub fn into_arc(self) -> DynCache {
            Arc::new(self)
        }
    }
}

pub use redis_backend::RedisCache;
