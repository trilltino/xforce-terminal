use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn, error};
use serde::{Deserialize, Serialize};

use super::{
    registry::{ContractRegistry, ContractMetadata, create_default_registry},
    queue::{ContractQueue, ContractOperation, OperationPriority, QueueResult},
    pool::PoolConfig,
    circuit_breaker::CircuitBreakerConfig,
};
use crate::error::{AppError, Result};
use crate::types::ContractFunction;

/// High-level contract manager that orchestrates all scalability components
pub struct ScalableContractManager {
    registry: Arc<ContractRegistry>,
    queue: Arc<ContractQueue>,
    metrics: Arc<tokio::sync::RwLock<ContractMetrics>>,
}

impl ScalableContractManager {
    /// Create a new scalable contract manager with all features enabled
    pub async fn new() -> Result<Self> {
        info!("🚀 Initializing Scalable Contract Manager");

        // Create contract registry with default contracts
        let registry = create_default_registry()
            .await
            .map_err(|e| AppError::Config(format!("Failed to create registry: {}", e)))?;

        // Create async queue for operations
        let queue = Arc::new(ContractQueue::new());

        // Initialize metrics
        let metrics = Arc::new(tokio::sync::RwLock::new(ContractMetrics::default()));

        // Start background tasks
        Self::start_background_tasks(queue.clone(), metrics.clone());

        info!("✅ Scalable Contract Manager initialized successfully");

        Ok(Self {
            registry,
            queue,
            metrics,
        })
    }

    /// Generate XDR for contract function with all scalability features
    pub async fn generate_xdr(
        &self,
        contract_id: &str,
        source_account: &str,
        function: &ContractFunction,
    ) -> Result<String> {
        // Get contract handle from registry
        let handle = self
            .registry
            .get(contract_id)
            .await
            .ok_or_else(|| AppError::Config(format!("Contract not found: {}", contract_id)))?;

        // Check cache first
        let cache_key = format!("xdr:{}:{}:{}", contract_id, source_account, function.name());
        if let Some(cached_xdr) = handle.cache.get(&cache_key).await {
            info!("✅ XDR retrieved from cache");
            self.record_cache_hit().await;
            return String::from_utf8(cached_xdr)
                .map_err(|e| AppError::XdrEncoding(format!("Invalid cached XDR: {}", e)));
        }

        self.record_cache_miss().await;

        // Get RPC connection from pool (validates connection is available)
        let _connection = handle
            .get_rpc_connection()
            .await
            .map_err(|e| AppError::StellarRpc(e))?;

        // Execute with circuit breaker protection
        let xdr_result = handle
            .call_with_protection(async {
                // Call the actual XDR generation
                super::client::generate_leaderboard_xdr(
                    &crate::services::stellar::XdrConfig {
                        contract_id: handle.metadata.contract_id.clone(),
                        network_passphrase: handle.metadata.network_passphrase.clone(),
                        rpc_url: handle.metadata.rpc_url.clone(),
                    },
                    source_account,
                    function,
                )
                .await
            })
            .await
            .map_err(|e| AppError::StellarRpc(e))?;

        // Cache the result (1 minute TTL for XDR)
        handle
            .cache
            .set(cache_key, xdr_result.clone().into_bytes(), Some(Duration::from_secs(60)))
            .await;

        self.record_xdr_generated().await;
        Ok(xdr_result)
    }

    /// Submit signed transaction via async queue with retry logic
    pub async fn submit_transaction(
        &self,
        contract_id: &str,
        source_account: String,
        function: ContractFunction,
        signed_xdr: String,
        priority: Option<OperationPriority>,
    ) -> Result<String> {
        // Create operation
        let operation = ContractOperation::new(
            contract_id.to_string(),
            function.name().to_string(),
            source_account,
            Some(signed_xdr),
        )
        .with_priority(priority.unwrap_or(OperationPriority::Normal))
        .with_max_retries(3);

        // Submit to queue
        let operation_id = self
            .queue
            .submit(operation)
            .await
            .map_err(|e| AppError::Transaction(e))?;

        self.record_transaction_submitted().await;
        Ok(operation_id)
    }

    /// Get operation result from queue
    pub async fn get_operation_result(&self) -> Option<QueueResult> {
        self.queue.next_result().await
    }

    /// Register a new contract dynamically
    pub async fn register_contract(&self, metadata: ContractMetadata) -> Result<()> {
        self.registry
            .register(metadata)
            .await
            .map_err(|e| AppError::Config(e))
    }

    /// List all registered contracts
    pub async fn list_contracts(&self) -> Vec<ContractMetadata> {
        self.registry.list_all().await
    }

    /// Get comprehensive system metrics
    pub async fn get_metrics(&self) -> ContractMetrics {
        self.metrics.read().await.clone()
    }

    /// Get detailed contract information
    pub async fn get_contract_info(&self, contract_id: &str) -> Result<ContractInfo> {
        let handle = self
            .registry
            .get(contract_id)
            .await
            .ok_or_else(|| AppError::Config(format!("Contract not found: {}", contract_id)))?;

        Ok(ContractInfo {
            metadata: handle.metadata.clone(),
            pool_stats: handle.rpc_pool.stats().await,
            circuit_breaker_stats: handle.circuit_breaker.stats().await,
            cache_stats: handle.cache.stats().await,
        })
    }

    /// Health check for the contract manager
    pub async fn health_check(&self) -> HealthStatus {
        let metrics = self.metrics.read().await.clone();
        let registry_stats = self.registry.stats().await;

        HealthStatus {
            healthy: registry_stats.enabled_contracts > 0,
            total_contracts: registry_stats.total_contracts,
            enabled_contracts: registry_stats.enabled_contracts,
            total_operations: metrics.total_operations,
            failed_operations: metrics.failed_operations,
            cache_hit_rate: metrics.cache_hit_rate(),
        }
    }

    /// Query contract events with filtering and pagination
    pub async fn query_events(
        &self,
        contract_id: &str,
        pagination: super::events::Pagination,
        filters: Vec<super::events::EventFilter>,
        limit: Option<u32>,
    ) -> Result<super::events::GetEventsResponse> {
        info!("[MANAGER] query_events called for contract: {}", contract_id);

        // Get contract handle
        let handle = self
            .registry
            .get(contract_id)
            .await
            .ok_or_else(|| AppError::Config(format!("Contract not found: {}", contract_id)))?;

        // Check cache first for recent events (optional optimization)
        let cache_key = format!("events:{}:{:?}:{:?}", contract_id, pagination, limit);
        if let Some(cached_response) = handle.cache.get(&cache_key).await {
            info!("[MANAGER] ✅ Events retrieved from cache");
            self.record_cache_hit().await;

            // Try to deserialize cached response
            if let Ok(events_response) = serde_json::from_slice::<super::events::GetEventsResponse>(&cached_response) {
                return Ok(events_response);
            }
        }

        self.record_cache_miss().await;

        // Get RPC connection from pool
        let _connection = handle
            .get_rpc_connection()
            .await
            .map_err(|e| AppError::StellarRpc(e))?;

        // Execute with circuit breaker protection
        let events_result = handle
            .call_with_protection(async {
                // Call get_events RPC method
                super::client::get_events(
                    &crate::services::stellar::XdrConfig {
                        contract_id: handle.metadata.contract_id.clone(),
                        network_passphrase: handle.metadata.network_passphrase.clone(),
                        rpc_url: handle.metadata.rpc_url.clone(),
                    },
                    pagination,
                    filters,
                    limit,
                )
                .await
            })
            .await
            .map_err(|e| AppError::StellarRpc(e))?;

        // Cache the result (30 seconds TTL for events - they change frequently)
        if let Ok(cached_bytes) = serde_json::to_vec(&events_result) {
            handle
                .cache
                .set(cache_key, cached_bytes, Some(std::time::Duration::from_secs(30)))
                .await;
        }

        // Update metrics
        let mut metrics = self.metrics.write().await;
        metrics.total_operations += 1;
        metrics.successful_operations += 1;

        info!("[MANAGER] ✅ query_events successful - {} events returned", events_result.event_count());

        Ok(events_result)
    }

    /// Simulate a transaction before submitting it to the network
    ///
    /// This method simulates a transaction without actually submitting it, using:
    /// - Cache for repeated simulations (60-second TTL)
    /// - Circuit breaker for fault tolerance
    /// - Connection pool for scalability
    /// - Metrics tracking
    ///
    /// # Arguments
    /// * `contract_id` - The contract ID to simulate against
    /// * `transaction_xdr` - Base64-encoded transaction envelope XDR
    /// * `options` - Optional simulation options
    ///
    /// # Returns
    /// A `SimulateTransactionResponse` containing simulation results
    pub async fn simulate_transaction(
        &self,
        contract_id: &str,
        transaction_xdr: &str,
        options: Option<super::simulation::SimulationOptions>,
    ) -> Result<super::simulation::SimulateTransactionResponse> {
        info!("[MANAGER] simulate_transaction called for contract: {}", contract_id);

        // Get contract handle
        let handle = self.registry.get(contract_id).await
            .ok_or_else(|| AppError::Config(format!("Contract not found: {}", contract_id)))?;

        // Check cache first (60-second TTL for simulations)
        let cache_key = format!("sim:{}:{}", contract_id, transaction_xdr);
        if let Some(cached_response) = handle.cache.get(&cache_key).await {
            self.record_cache_hit().await;
            if let Ok(sim_response) = serde_json::from_slice(&cached_response) {
                info!("[MANAGER] ✅ Using cached simulation result");
                return Ok(sim_response);
            }
        }

        self.record_cache_miss().await;

        // Get RPC connection from pool
        let _connection = handle.get_rpc_connection().await
            .map_err(|e| AppError::Internal(format!("Failed to get RPC connection: {}", e)))?;

        // Build contract config
        let config = crate::services::stellar::XdrConfig {
            contract_id: handle.metadata.contract_id.clone(),
            rpc_url: handle.metadata.rpc_url.clone(),
            network_passphrase: handle.metadata.network_passphrase.clone(),
        };

        // Execute with circuit breaker protection
        let simulation_result = handle
            .call_with_protection(async {
                super::client::simulate_transaction(&config, transaction_xdr, options).await
            })
            .await
            .map_err(|e| AppError::Internal(format!("Circuit breaker error: {}", e)))?;

        // Cache result (60 seconds TTL - longer than events since simulations are more expensive)
        if let Ok(cached_bytes) = serde_json::to_vec(&simulation_result) {
            handle.cache.set(
                cache_key,
                cached_bytes,
                Some(std::time::Duration::from_secs(60))
            ).await;
        }

        // Update metrics
        let mut metrics = self.metrics.write().await;
        metrics.total_operations += 1;
        metrics.successful_operations += 1;

        if simulation_result.is_success() {
            info!(
                "[MANAGER] ✅ simulate_transaction successful - fee: {:?}",
                simulation_result.min_resource_fee
            );
        } else {
            info!(
                "[MANAGER] ⚠️  simulate_transaction returned error: {:?}",
                simulation_result.error
            );
        }

        Ok(simulation_result)
    }

    /// Get contract storage data
    ///
    /// This method reads contract storage directly without executing a transaction, using:
    /// - Cache for repeated reads (5-minute TTL - longer since state changes less frequently)
    /// - Circuit breaker for fault tolerance
    /// - Connection pool for scalability
    /// - Metrics tracking
    ///
    /// # Arguments
    /// * `contract_id` - The contract ID to query
    /// * `key` - Storage key as base64 XDR encoded ScVal
    /// * `durability` - Storage durability (Temporary or Persistent)
    ///
    /// # Returns
    /// A `LedgerEntryResult` containing the storage data and TTL information
    pub async fn get_contract_data(
        &self,
        contract_id: &str,
        key: &str,
        durability: super::state::Durability,
    ) -> Result<super::state::LedgerEntryResult> {
        info!(
            "[MANAGER] get_contract_data called for contract: {}, durability: {:?}",
            contract_id, durability
        );

        // Get contract handle
        let handle = self.registry.get(contract_id).await
            .ok_or_else(|| AppError::Config(format!("Contract not found: {}", contract_id)))?;

        // Check cache first (5-minute TTL for contract state)
        let cache_key = format!("state:{}:{}:{:?}", contract_id, key, durability);
        if let Some(cached_response) = handle.cache.get(&cache_key).await {
            self.record_cache_hit().await;
            if let Ok(entry) = serde_json::from_slice(&cached_response) {
                info!("[MANAGER] ✅ Using cached contract data");
                return Ok(entry);
            }
        }

        self.record_cache_miss().await;

        // Get RPC connection from pool
        let _connection = handle.get_rpc_connection().await
            .map_err(|e| AppError::Internal(format!("Failed to get RPC connection: {}", e)))?;

        // Build contract config
        let config = crate::services::stellar::XdrConfig {
            contract_id: handle.metadata.contract_id.clone(),
            rpc_url: handle.metadata.rpc_url.clone(),
            network_passphrase: handle.metadata.network_passphrase.clone(),
        };

        // Execute with circuit breaker protection
        let data_result = handle
            .call_with_protection(async {
                super::client::get_contract_data(&config, contract_id, key, durability).await
            })
            .await
            .map_err(|e| AppError::Internal(format!("Circuit breaker error: {}", e)))?;

        // Cache result (5 minutes TTL - contract state changes less frequently)
        if let Ok(cached_bytes) = serde_json::to_vec(&data_result) {
            handle.cache.set(
                cache_key,
                cached_bytes,
                Some(std::time::Duration::from_secs(300))
            ).await;
        }

        // Update metrics
        let mut metrics = self.metrics.write().await;
        metrics.total_operations += 1;
        metrics.successful_operations += 1;

        info!("[MANAGER] ✅ get_contract_data successful");

        Ok(data_result)
    }

    /// Call a generic contract function (read-only via simulation)
    ///
    /// This method allows calling ANY Soroban contract function by:
    /// - Building a transaction XDR for the function call
    /// - Simulating the transaction (read-only, no fees)
    /// - Parsing the result from XDR to JSON
    ///
    /// Uses scalability features:
    /// - Cache for repeated function calls (60-second TTL)
    /// - Circuit breaker for fault tolerance
    /// - Connection pool for scalability
    /// - Metrics tracking
    ///
    /// # Arguments
    /// * `contract_id` - Contract address to call
    /// * `function_name` - Name of the function to invoke
    /// * `parameters` - Function parameters (will be converted to ScVal)
    /// * `source_account` - Optional source account (uses default if None)
    ///
    /// # Returns
    /// A `CallContractFunctionResponse` containing the parsed result and simulation details
    pub async fn call_contract_function(
        &self,
        contract_id: &str,
        function_name: &str,
        parameters: Vec<shared::dto::soroban::FunctionParameter>,
        source_account: Option<&str>,
    ) -> Result<shared::dto::soroban::CallContractFunctionResponse> {
        info!(
            "[MANAGER] call_contract_function - contract: {}, function: {}",
            contract_id, function_name
        );

        // Get contract handle
        let handle = self.registry.get(contract_id).await
            .ok_or_else(|| AppError::Config(format!("Contract not found: {}", contract_id)))?;

        // Build cache key from function name and parameters
        let params_hash = format!("{:?}", parameters);
        let cache_key = format!("func:{}:{}:{}", contract_id, function_name, params_hash);

        // Check cache first (60-second TTL for function calls)
        if let Some(cached_response) = handle.cache.get(&cache_key).await {
            self.record_cache_hit().await;
            if let Ok(func_response) = serde_json::from_slice(&cached_response) {
                info!("[MANAGER] ✅ Using cached function call result");
                return Ok(func_response);
            }
        }

        self.record_cache_miss().await;

        // Get RPC connection from pool
        let _connection = handle.get_rpc_connection().await
            .map_err(|e| AppError::Internal(format!("Failed to get RPC connection: {}", e)))?;

        // Execute with circuit breaker protection
        let func_result = handle
            .call_with_protection(async {
                super::client::call_contract_function(
                    contract_id,
                    function_name,
                    parameters,
                    source_account,
                    &handle.metadata.rpc_url,
                    &handle.metadata.network_passphrase,
                ).await
            })
            .await
            .map_err(|e| AppError::Internal(format!("Circuit breaker error: {}", e)))?;

        // Cache successful results (60 seconds TTL)
        if func_result.success {
            if let Ok(cached_bytes) = serde_json::to_vec(&func_result) {
                handle.cache.set(
                    cache_key,
                    cached_bytes,
                    Some(std::time::Duration::from_secs(60))
                ).await;
            }
        }

        // Update metrics
        let mut metrics = self.metrics.write().await;
        metrics.total_operations += 1;
        if func_result.success {
            metrics.successful_operations += 1;
        } else {
            metrics.failed_operations += 1;
        }

        if func_result.success {
            info!("[MANAGER] ✅ call_contract_function successful");
        } else {
            info!("[MANAGER] ⚠️ call_contract_function failed: {:?}", func_result.error);
        }

        Ok(func_result)
    }

    // Internal metric recording methods
    async fn record_cache_hit(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.cache_hits += 1;
    }

    async fn record_cache_miss(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.cache_misses += 1;
    }

    async fn record_xdr_generated(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.xdr_generated += 1;
        metrics.total_operations += 1;
    }

    async fn record_transaction_submitted(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.transactions_submitted += 1;
        metrics.total_operations += 1;
    }

    /// Start background tasks for queue processing, cache cleanup, etc.
    fn start_background_tasks(
        queue: Arc<ContractQueue>,
        metrics: Arc<tokio::sync::RwLock<ContractMetrics>>,
    ) {
        // Queue result processor
        tokio::spawn(async move {
            info!("🔄 Starting queue result processor");

            while let Some(result) = queue.next_result().await {
                match result {
                    QueueResult::Success { operation_id, result } => {
                        info!("✅ Operation {} succeeded: {}", operation_id, result);
                        let mut m = metrics.write().await;
                        m.successful_operations += 1;
                    }
                    QueueResult::Retry { operation_id, attempt } => {
                        warn!("🔄 Operation {} retry attempt {}", operation_id, attempt);
                        let mut m = metrics.write().await;
                        m.retried_operations += 1;
                    }
                    QueueResult::Failed { operation_id, error } => {
                        error!("❌ Operation {} failed: {}", operation_id, error);
                        let mut m = metrics.write().await;
                        m.failed_operations += 1;
                    }
                }
            }
        });
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContractMetrics {
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub retried_operations: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub xdr_generated: u64,
    pub transactions_submitted: u64,
}

impl ContractMetrics {
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            return 0.0;
        }
        (self.cache_hits as f64 / total as f64) * 100.0
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_operations == 0 {
            return 0.0;
        }
        (self.successful_operations as f64 / self.total_operations as f64) * 100.0
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ContractInfo {
    pub metadata: ContractMetadata,
    pub pool_stats: super::pool::PoolStats,
    pub circuit_breaker_stats: super::circuit_breaker::CircuitBreakerStats,
    pub cache_stats: super::cache::CacheStats,
}

#[derive(Debug, Clone, Serialize)]
pub struct HealthStatus {
    pub healthy: bool,
    pub total_contracts: usize,
    pub enabled_contracts: usize,
    pub total_operations: u64,
    pub failed_operations: u64,
    pub cache_hit_rate: f64,
}

/// Configuration for the scalable contract manager
#[derive(Debug, Clone)]
pub struct ContractManagerConfig {
    pub pool_config: PoolConfig,
    pub circuit_breaker_config: CircuitBreakerConfig,
    pub cache_ttl: Duration,
}

impl Default for ContractManagerConfig {
    fn default() -> Self {
        Self {
            pool_config: PoolConfig::default(),
            circuit_breaker_config: CircuitBreakerConfig::default(),
            cache_ttl: Duration::from_secs(300),
        }
    }
}

// Conversion implementations for shared DTOs
impl From<ContractMetrics> for shared::dto::soroban::ContractMetrics {
    fn from(m: ContractMetrics) -> Self {
        Self {
            total_operations: m.total_operations,
            successful_operations: m.successful_operations,
            failed_operations: m.failed_operations,
            retried_operations: m.retried_operations,
            cache_hits: m.cache_hits,
            cache_misses: m.cache_misses,
            xdr_generated: m.xdr_generated,
            transactions_submitted: m.transactions_submitted,
        }
    }
}

impl From<HealthStatus> for shared::dto::soroban::HealthStatus {
    fn from(h: HealthStatus) -> Self {
        Self {
            healthy: h.healthy,
            total_contracts: h.total_contracts,
            enabled_contracts: h.enabled_contracts,
            total_operations: h.total_operations,
            failed_operations: h.failed_operations,
            cache_hit_rate: h.cache_hit_rate,
        }
    }
}

impl From<ContractInfo> for shared::dto::soroban::ContractInfo {
    fn from(i: ContractInfo) -> Self {
        Self {
            metadata: i.metadata.into(),
            pool_stats: i.pool_stats.into(),
            circuit_breaker_stats: i.circuit_breaker_stats.into(),
            cache_stats: i.cache_stats.into(),
        }
    }
}

impl From<ContractMetadata> for shared::dto::soroban::ContractMetadata {
    fn from(m: ContractMetadata) -> Self {
        Self {
            contract_id: m.contract_id,
            name: m.name,
            network: m.network.into(),
            network_passphrase: m.network_passphrase,
            rpc_url: m.rpc_url,
            description: m.description,
            version: m.version,
            enabled: m.enabled,
        }
    }
}

impl From<super::registry::NetworkType> for shared::dto::soroban::NetworkType {
    fn from(n: super::registry::NetworkType) -> Self {
        match n {
            super::registry::NetworkType::Testnet => Self::Testnet,
            super::registry::NetworkType::Mainnet => Self::Mainnet,
            super::registry::NetworkType::Futurenet => Self::Futurenet,
            super::registry::NetworkType::Standalone => Self::Standalone,
        }
    }
}

impl From<super::pool::PoolStats> for shared::dto::soroban::PoolStats {
    fn from(p: super::pool::PoolStats) -> Self {
        Self {
            total_connections: p.total_connections,
            max_connections: p.max_connections,
            available: p.available,
        }
    }
}

impl From<super::circuit_breaker::CircuitBreakerStats> for shared::dto::soroban::CircuitBreakerStats {
    fn from(c: super::circuit_breaker::CircuitBreakerStats) -> Self {
        Self {
            state: c.state.into(),
            failure_count: c.failure_count,
            success_count: c.success_count,
            is_open: c.is_open,
        }
    }
}

impl From<super::circuit_breaker::CircuitState> for shared::dto::soroban::CircuitState {
    fn from(s: super::circuit_breaker::CircuitState) -> Self {
        match s {
            super::circuit_breaker::CircuitState::Closed => Self::Closed,
            super::circuit_breaker::CircuitState::Open => Self::Open,
            super::circuit_breaker::CircuitState::HalfOpen => Self::HalfOpen,
        }
    }
}

impl From<super::cache::CacheStats> for shared::dto::soroban::CacheStats {
    fn from(c: super::cache::CacheStats) -> Self {
        Self {
            total_entries: c.total_entries,
            expired_entries: c.expired_entries,
            active_entries: c.active_entries,
        }
    }
}
