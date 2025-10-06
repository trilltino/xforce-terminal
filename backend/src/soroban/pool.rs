use soroban_client::{Server, Options};
use std::sync::Arc;
use tokio::sync::{Semaphore, RwLock};
use tracing::info;
use std::time::{Duration, Instant};
use crate::error::{AppError, Result};

/// Connection pool configuration
#[derive(Debug, Clone)]
pub struct PoolConfig {
    pub max_connections: usize,
    pub idle_timeout: Duration,
    pub connection_timeout: Duration,
    pub max_retries: u32,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 50,
            idle_timeout: Duration::from_secs(300),
            connection_timeout: Duration::from_secs(30),
            max_retries: 3,
        }
    }
}

/// Pooled RPC connection
#[derive(Clone)]
struct PooledConnection {
    server: Arc<Server>,
    last_used: Instant,
}

/// RPC Connection Pool for massive scalability
pub struct StellarRpcPool {
    rpc_url: String,
    connections: Arc<RwLock<Vec<PooledConnection>>>,
    semaphore: Arc<Semaphore>,
    config: PoolConfig,
}

impl StellarRpcPool {
    pub fn new(rpc_url: String, config: PoolConfig) -> Result<Self> {
        info!("🏊 Initializing Stellar RPC pool with {} connections", config.max_connections);

        let semaphore = Arc::new(Semaphore::new(config.max_connections));
        let connections = Arc::new(RwLock::new(Vec::with_capacity(config.max_connections)));

        Ok(Self {
            rpc_url,
            connections,
            semaphore,
            config,
        })
    }

    /// Get a connection from the pool (or create new if needed)
    pub async fn get_connection(&self) -> Result<PooledRpcConnection> {
        // Acquire semaphore permit (limits concurrent connections)
        let permit = self.semaphore.clone()
            .acquire_owned()
            .await
            .map_err(|e| AppError::StellarRpc(format!("Failed to acquire connection: {}", e)))?;

        // Try to get existing connection
        let mut connections = self.connections.write().await;

        // Remove stale connections
        connections.retain(|conn| {
            conn.last_used.elapsed() < self.config.idle_timeout
        });

        // Reuse existing connection or create new
        let pooled_conn = if let Some(conn) = connections.pop() {
            conn
        } else {
            info!("📡 Creating new RPC connection to {}", self.rpc_url);
            let server = Server::new(&self.rpc_url, Options::default())
                .map_err(|e| AppError::StellarRpc(format!("Failed to create RPC server: {:?}", e)))?;

            PooledConnection {
                server: Arc::new(server),
                last_used: Instant::now(),
            }
        };

        Ok(PooledRpcConnection {
            connection: pooled_conn,
            pool: self.connections.clone(),
            _permit: permit,
        })
    }

    /// Get pool statistics
    pub async fn stats(&self) -> PoolStats {
        let connections = self.connections.read().await;
        PoolStats {
            total_connections: connections.len(),
            max_connections: self.config.max_connections,
            available: self.semaphore.available_permits(),
        }
    }
}

/// RAII wrapper for pooled connection - automatically returns to pool
pub struct PooledRpcConnection {
    connection: PooledConnection,
    pool: Arc<RwLock<Vec<PooledConnection>>>,
    _permit: tokio::sync::OwnedSemaphorePermit,
}

impl PooledRpcConnection {
    pub fn server(&self) -> &Server {
        &self.connection.server
    }
}

impl Drop for PooledRpcConnection {
    fn drop(&mut self) {
        let conn = PooledConnection {
            server: self.connection.server.clone(),
            last_used: Instant::now(),
        };

        let pool = self.pool.clone();
        tokio::spawn(async move {
            pool.write().await.push(conn);
        });
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PoolStats {
    pub total_connections: usize,
    pub max_connections: usize,
    pub available: usize,
}
