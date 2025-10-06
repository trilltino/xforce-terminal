use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Cache entry with expiration
#[derive(Clone, Debug)]
struct CacheEntry<T> {
    value: T,
    expires_at: Instant,
}

/// High-performance in-memory cache for contract state
/// Can be extended to use Redis for distributed caching
pub struct ContractCache<T>
where
    T: Clone + Send + Sync,
{
    store: Arc<RwLock<HashMap<String, CacheEntry<T>>>>,
    default_ttl: Duration,
}

impl<T> ContractCache<T>
where
    T: Clone + Send + Sync,
{
    pub fn new(default_ttl: Duration) -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            default_ttl,
        }
    }

    /// Get value from cache
    pub async fn get(&self, key: &str) -> Option<T> {
        let store = self.store.read().await;

        if let Some(entry) = store.get(key) {
            if Instant::now() < entry.expires_at {
                debug!("✅ Cache HIT for key: {}", key);
                return Some(entry.value.clone());
            } else {
                debug!("⏰ Cache EXPIRED for key: {}", key);
            }
        } else {
            debug!("❌ Cache MISS for key: {}", key);
        }

        None
    }

    /// Set value in cache with custom TTL
    pub async fn set(&self, key: String, value: T, ttl: Option<Duration>) {
        let ttl = ttl.unwrap_or(self.default_ttl);
        let mut store = self.store.write().await;

        store.insert(
            key.clone(),
            CacheEntry {
                value,
                expires_at: Instant::now() + ttl,
            },
        );

        debug!("💾 Cache SET for key: {} (TTL: {:?})", key, ttl);
    }

    /// Invalidate cache entry
    pub async fn invalidate(&self, key: &str) {
        let mut store = self.store.write().await;
        if store.remove(key).is_some() {
            debug!("🗑️  Cache INVALIDATED for key: {}", key);
        }
    }

    /// Clear all expired entries
    pub async fn cleanup_expired(&self) -> usize {
        let mut store = self.store.write().await;
        let now = Instant::now();
        let initial_count = store.len();

        store.retain(|_, entry| entry.expires_at > now);

        let removed = initial_count - store.len();
        if removed > 0 {
            info!("🧹 Cleaned up {} expired cache entries", removed);
        }
        removed
    }

    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        let store = self.store.read().await;
        let now = Instant::now();

        let expired = store.values().filter(|e| e.expires_at <= now).count();

        CacheStats {
            total_entries: store.len(),
            expired_entries: expired,
            active_entries: store.len() - expired,
        }
    }

    /// Clear all cache entries
    pub async fn clear(&self) {
        let mut store = self.store.write().await;
        let count = store.len();
        store.clear();
        info!("🗑️  Cleared {} cache entries", count);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_entries: usize,
    pub expired_entries: usize,
    pub active_entries: usize,
}

/// Specialized cache for leaderboard data
pub type LeaderboardCache = ContractCache<Vec<u8>>;

/// Specialized cache for contract function results
pub type FunctionResultCache = ContractCache<String>;

/// Start background cleanup task
pub fn start_cache_cleanup_task<T>(cache: Arc<ContractCache<T>>)
where
    T: Clone + Send + Sync + 'static,
{
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));

        loop {
            interval.tick().await;
            cache.cleanup_expired().await;
        }
    });
}
