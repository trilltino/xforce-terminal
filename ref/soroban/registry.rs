use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

use super::pool::{StellarRpcPool, PoolConfig};
use super::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
use super::cache::ContractCache;

/// Contract metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractMetadata {
    pub contract_id: String,
    pub name: String,
    pub network: NetworkType,
    pub network_passphrase: String,
    pub rpc_url: String,
    pub description: Option<String>,
    pub version: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkType {
    Testnet,
    Mainnet,
    Futurenet,
    Standalone,
}

impl NetworkType {
    pub fn default_passphrase(&self) -> &'static str {
        match self {
            NetworkType::Testnet => "Test SDF Network ; September 2015",
            NetworkType::Mainnet => "Public Global Stellar Network ; September 2015",
            NetworkType::Futurenet => "Test SDF Future Network ; October 2022",
            NetworkType::Standalone => "Standalone Network ; February 2017",
        }
    }

    pub fn default_rpc_url(&self) -> &'static str {
        match self {
            NetworkType::Testnet => "https://soroban-testnet.stellar.org",
            NetworkType::Mainnet => "https://mainnet.sorobanrpc.com",
            NetworkType::Futurenet => "https://rpc-futurenet.stellar.org",
            NetworkType::Standalone => "http://localhost:8000/soroban/rpc",
        }
    }
}

/// Per-contract resources
struct ContractResources {
    metadata: ContractMetadata,
    rpc_pool: Arc<StellarRpcPool>,
    circuit_breaker: Arc<CircuitBreaker>,
    cache: Arc<ContractCache<Vec<u8>>>,
}

/// Multi-contract registry for massive scale
pub struct ContractRegistry {
    contracts: Arc<RwLock<HashMap<String, ContractResources>>>,
    default_pool_config: PoolConfig,
    default_circuit_config: CircuitBreakerConfig,
}

impl ContractRegistry {
    pub fn new(
        pool_config: Option<PoolConfig>,
        circuit_config: Option<CircuitBreakerConfig>,
    ) -> Self {
        info!("🗂️  Initializing contract registry");

        Self {
            contracts: Arc::new(RwLock::new(HashMap::new())),
            default_pool_config: pool_config.unwrap_or_default(),
            default_circuit_config: circuit_config.unwrap_or_default(),
        }
    }

    /// Register a new contract
    pub async fn register(&self, metadata: ContractMetadata) -> Result<(), String> {
        if !metadata.enabled {
            warn!("⚠️  Contract {} is disabled, skipping registration", metadata.contract_id);
            return Ok(());
        }

        info!("📝 Registering contract: {} ({})", metadata.name, metadata.contract_id);

        // Create RPC pool
        let rpc_pool = StellarRpcPool::new(
            metadata.rpc_url.clone(),
            self.default_pool_config.clone(),
        )
        .map_err(|e| format!("Failed to create RPC pool: {}", e))?;

        // Create circuit breaker
        let circuit_breaker = CircuitBreaker::new(self.default_circuit_config.clone());

        // Create cache (5 minute TTL)
        let cache = ContractCache::new(std::time::Duration::from_secs(300));

        let resources = ContractResources {
            metadata: metadata.clone(),
            rpc_pool: Arc::new(rpc_pool),
            circuit_breaker: Arc::new(circuit_breaker),
            cache: Arc::new(cache),
        };

        let mut contracts = self.contracts.write().await;
        contracts.insert(metadata.contract_id.clone(), resources);

        info!("✅ Contract {} registered successfully", metadata.contract_id);
        Ok(())
    }

    /// Get contract by ID
    pub async fn get(&self, contract_id: &str) -> Option<ContractHandle> {
        let contracts = self.contracts.read().await;

        contracts.get(contract_id).map(|resources| ContractHandle {
            metadata: resources.metadata.clone(),
            rpc_pool: resources.rpc_pool.clone(),
            circuit_breaker: resources.circuit_breaker.clone(),
            cache: resources.cache.clone(),
        })
    }

    /// List all registered contracts
    pub async fn list_all(&self) -> Vec<ContractMetadata> {
        let contracts = self.contracts.read().await;
        contracts
            .values()
            .map(|r| r.metadata.clone())
            .collect()
    }

    /// Unregister a contract
    pub async fn unregister(&self, contract_id: &str) -> Result<(), String> {
        let mut contracts = self.contracts.write().await;

        if contracts.remove(contract_id).is_some() {
            info!("🗑️  Unregistered contract: {}", contract_id);
            Ok(())
        } else {
            Err(format!("Contract not found: {}", contract_id))
        }
    }

    /// Get registry statistics
    pub async fn stats(&self) -> RegistryStats {
        let contracts = self.contracts.read().await;

        let enabled_count = contracts
            .values()
            .filter(|r| r.metadata.enabled)
            .count();

        RegistryStats {
            total_contracts: contracts.len(),
            enabled_contracts: enabled_count,
            disabled_contracts: contracts.len() - enabled_count,
        }
    }
}

/// Handle to interact with a specific contract
#[derive(Clone)]
pub struct ContractHandle {
    pub metadata: ContractMetadata,
    pub rpc_pool: Arc<StellarRpcPool>,
    pub circuit_breaker: Arc<CircuitBreaker>,
    pub cache: Arc<ContractCache<Vec<u8>>>,
}

impl ContractHandle {
    /// Get a pooled RPC connection
    pub async fn get_rpc_connection(&self) -> Result<super::pool::PooledRpcConnection, String> {
        self.rpc_pool
            .get_connection()
            .await
            .map_err(|e| format!("Failed to get RPC connection: {}", e))
    }

    /// Execute function with circuit breaker protection
    pub async fn call_with_protection<F, T, E>(&self, f: F) -> Result<T, String>
    where
        F: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display,
    {
        self.circuit_breaker
            .call(f)
            .await
            .map_err(|e| format!("Circuit breaker error: {}", e))
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RegistryStats {
    pub total_contracts: usize,
    pub enabled_contracts: usize,
    pub disabled_contracts: usize,
}

/// Create default contract registry with common contracts
pub async fn create_default_registry() -> Result<Arc<ContractRegistry>, String> {
    let registry = Arc::new(ContractRegistry::new(None, None));

    // Register leaderboard contract
    let leaderboard_metadata = ContractMetadata {
        contract_id: std::env::var("CONTRACT_ID")
            .unwrap_or_else(|_| "CC25DOXDMJ3OMDKE4ZETPY34734VQABAYAXSPKFXJ7I2STLCFV2VT7FC".to_string()),
        name: "Stellar Heads Leaderboard".to_string(),
        network: NetworkType::Testnet,
        network_passphrase: NetworkType::Testnet.default_passphrase().to_string(),
        rpc_url: std::env::var("SOROBAN_RPC_URL")
            .unwrap_or_else(|_| NetworkType::Testnet.default_rpc_url().to_string()),
        description: Some("Game leaderboard smart contract".to_string()),
        version: Some("1.0.0".to_string()),
        enabled: true,
    };

    registry.register(leaderboard_metadata).await?;

    // Register Reflector Oracle (Testnet)
    let reflector_testnet_metadata = ContractMetadata {
        contract_id: "CAVLP5DH2GJPZMVO7IJY4CVOD5MWEFTJFVPD2YY2FQXOQHRGHK4D6HLP".to_string(),
        name: "Reflector Oracle (Testnet)".to_string(),
        network: NetworkType::Testnet,
        network_passphrase: NetworkType::Testnet.default_passphrase().to_string(),
        rpc_url: std::env::var("SOROBAN_RPC_URL")
            .unwrap_or_else(|_| NetworkType::Testnet.default_rpc_url().to_string()),
        description: Some("Reflector price oracle for Stellar - SEP-40 compatible".to_string()),
        version: Some("1.0.0".to_string()),
        enabled: true,
    };

    registry.register(reflector_testnet_metadata).await?;

    // Register Reflector FX Rates Oracle (Testnet)
    let reflector_fx_metadata = ContractMetadata {
        contract_id: "CCYOZJCOPG34LLQQ7N24YXBM7LL62R7ONMZ3G6WZAAYPB5OYKOMJRN63".to_string(),
        name: "Reflector FX Rates (Testnet)".to_string(),
        network: NetworkType::Testnet,
        network_passphrase: NetworkType::Testnet.default_passphrase().to_string(),
        rpc_url: std::env::var("SOROBAN_RPC_URL")
            .unwrap_or_else(|_| NetworkType::Testnet.default_rpc_url().to_string()),
        description: Some("Reflector FX rates oracle for fiat currencies".to_string()),
        version: Some("1.0.0".to_string()),
        enabled: true,
    };

    registry.register(reflector_fx_metadata).await?;

    // Register Reflector Oracle (Mainnet) - disabled by default for testnet setup
    let reflector_mainnet_metadata = ContractMetadata {
        contract_id: "CAFJZQWSED6YAWZU3GWRTOCNPPCGBN32L7QV43XX5LZLFTK6JLN34DLN".to_string(),
        name: "Reflector Oracle (Mainnet)".to_string(),
        network: NetworkType::Mainnet,
        network_passphrase: NetworkType::Mainnet.default_passphrase().to_string(),
        rpc_url: NetworkType::Mainnet.default_rpc_url().to_string(),
        description: Some("Reflector price oracle for Stellar - SEP-40 compatible (Mainnet)".to_string()),
        version: Some("1.0.0".to_string()),
        enabled: false, // Disabled by default since we're on testnet
    };

    registry.register(reflector_mainnet_metadata).await?;

    // Register Blend Protocol - Pool Factory V2 (Testnet)
    let blend_pool_factory_metadata = ContractMetadata {
        contract_id: "CDSMKKCWEAYQW4DAUSH3XGRMIVIJB44TZ3UA5YCRHT6MP4LWEWR4GYV6".to_string(),
        name: "Blend Pool Factory V2 (Testnet)".to_string(),
        network: NetworkType::Testnet,
        network_passphrase: NetworkType::Testnet.default_passphrase().to_string(),
        rpc_url: std::env::var("SOROBAN_RPC_URL")
            .unwrap_or_else(|_| NetworkType::Testnet.default_rpc_url().to_string()),
        description: Some("Blend lending protocol - Pool Factory for creating lending pools".to_string()),
        version: Some("2.0.0".to_string()),
        enabled: true,
    };

    registry.register(blend_pool_factory_metadata).await?;

    // Register Blend Protocol - Main Test Pool (Testnet)
    let blend_test_pool_metadata = ContractMetadata {
        contract_id: "CDDG7DLOWSHRYQ2HWGZEZ4UTR7LPTKFFHN3QUCSZEXOWOPARMONX6T65".to_string(),
        name: "Blend Test Pool (Testnet)".to_string(),
        network: NetworkType::Testnet,
        network_passphrase: NetworkType::Testnet.default_passphrase().to_string(),
        rpc_url: std::env::var("SOROBAN_RPC_URL")
            .unwrap_or_else(|_| NetworkType::Testnet.default_rpc_url().to_string()),
        description: Some("Blend lending protocol - Main test lending pool".to_string()),
        version: Some("2.0.0".to_string()),
        enabled: true,
    };

    registry.register(blend_test_pool_metadata).await?;

    // Register Blend Protocol - Backstop V2 (Testnet)
    let blend_backstop_metadata = ContractMetadata {
        contract_id: "CBHWKF4RHIKOKSURAKXSJRIIA7RJAMJH4VHRVPYGUF4AJ5L544LYZ35X".to_string(),
        name: "Blend Backstop V2 (Testnet)".to_string(),
        network: NetworkType::Testnet,
        network_passphrase: NetworkType::Testnet.default_passphrase().to_string(),
        rpc_url: std::env::var("SOROBAN_RPC_URL")
            .unwrap_or_else(|_| NetworkType::Testnet.default_rpc_url().to_string()),
        description: Some("Blend lending protocol - Backstop module for pool insurance".to_string()),
        version: Some("2.0.0".to_string()),
        enabled: true,
    };

    registry.register(blend_backstop_metadata).await?;

    Ok(registry)
}

#[derive(Debug, Clone, Serialize)]
pub struct ContractInfo {
    pub metadata: ContractMetadata,
    pub pool_stats: super::pool::PoolStats,
    pub circuit_breaker_stats: super::circuit_breaker::CircuitBreakerStats,
    pub cache_stats: super::cache::CacheStats,
}
