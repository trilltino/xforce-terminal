use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallContractFunctionRequest {
    pub contract_id: String,
    pub function_name: String,
    pub parameters: Vec<FunctionParameter>,
    pub source_account: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FunctionParameter {
    Symbol(String),
    Address(String),
    U32(u32),
    I32(i32),
    U64(u64),
    I64(i64),
    I128(i128),
    String(String),
    Bool(bool),
    Bytes(Vec<u8>),
    Vec(Vec<FunctionParameter>),
    Enum(String, Option<Box<FunctionParameter>>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallContractFunctionResponse {
    pub success: bool,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub result_xdr: Option<String>,
    pub simulation: Option<SimulationDetailsDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationDetailsDto {
    pub latest_ledger: Option<u32>,
    pub min_resource_fee: Option<String>,
    pub cpu_instructions: Option<u64>,
    pub events: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceCost {
    pub cpu_insns: u64,
    pub mem_bytes: u64,
}

// Metrics and stats DTOs
#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub healthy: bool,
    pub total_contracts: usize,
    pub enabled_contracts: usize,
    pub total_operations: u64,
    pub failed_operations: u64,
    pub cache_hit_rate: f64,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NetworkType {
    Testnet,
    Mainnet,
    Futurenet,
    Standalone,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractInfo {
    pub metadata: ContractMetadata,
    pub pool_stats: PoolStats,
    pub circuit_breaker_stats: CircuitBreakerStats,
    pub cache_stats: CacheStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStats {
    pub total_connections: usize,
    pub max_connections: usize,
    pub available: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerStats {
    pub state: CircuitState,
    pub failure_count: u32,
    pub success_count: u32,
    pub is_open: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_entries: usize,
    pub expired_entries: usize,
    pub active_entries: usize,
}
